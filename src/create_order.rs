use crate::big_decimal::{BigDecimal, WBalance};
use crate::*;
use near_sdk::env::current_account_id;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{ext_contract, is_promise_success, log, Gas};
use std::ops::Range;

mod contract_interfaces;

const NO_DEPOSIT: u128 = 0;
const GAS_FOR_BORROW: Gas = Gas(180_000_000_000_000);

pub const REF_FINANCE: &str = "ref-finance-101.testnet";

#[ext_contract(ext_market)]
trait MarketInterface {
    fn borrow(&mut self, amount: WBalance) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_token)]
trait NEP141Token {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: WBalance,
        memo: Option<String>,
        msg: String,
    );

    fn ft_transfer(&mut self, receiver_id: AccountId, amount: WBalance, memo: Option<String>);
}

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn swap_callback(
        &mut self,
        user: AccountId,
        amount: WBalance,
        order: Order,
    ) -> PromiseOrValue<Balance>;
}

/// Message parameters to receive via token function call.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
enum TokenReceiverMessage {
    /// Alternative to deposit + execute actions call.
    Execute {
        force: bool,
        /// List of sequential actions.
        actions: Vec<Action>,
    },
}

/// Single swap action.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    /// Pool which should be used for swapping.
    pub pool_id: u64,
    /// Token to swap from.
    pub token_in: AccountId,
    /// Amount to exchange.
    /// If amount_in is None, it will take amount_out from previous step.
    /// Will fail if amount_in is None on the first step.
    pub amount_in: Option<U128>,
    /// Token to swap into.
    pub token_out: AccountId,
    /// Required minimum amount of token_out.
    pub min_amount_out: U128,
}

/// Single action. Allows to execute sequence of various actions initiated by an account.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum Action {
    Swap(SwapAction),
}

#[near_bindgen]
impl Contract {
    pub fn create_order(
        &mut self,
        order_type: OrderType,
        mut amount: WBalance,
        sell_token: AccountId,
        buy_token: AccountId,
        leverage: U128,
    ) -> PromiseOrValue<WBalance> {
        let user = env::signer_account_id();

        require!(
            self.balance_of(user.clone(), sell_token.clone()) >= amount.0,
            "User doesn't have enough deposit to proceed this action"
        );

        let mut amount_to_proceed = amount;

        if BigDecimal::from(leverage) > BigDecimal::one() {
            let borrow_token_amount = U128::from(
                BigDecimal::from(amount)
                    * self.calculate_xrate(buy_token.clone(), sell_token.clone())
                    * BigDecimal::from(leverage),
            );
            log!("borrowing amount {}", borrow_token_amount.0);

            self.borrow_buy_token(borrow_token_amount, buy_token.clone());

            // if we have borrowed some tokens we have to add to liquidity pool corresponding amount
            amount_to_proceed = borrow_token_amount;
        }

        let min_amount_out = U128::from(
            BigDecimal::from(U128::from(amount_to_proceed))
                * self.calculate_xrate(buy_token.clone(), sell_token.clone()),
        );
        log!("min_amount_out {}", min_amount_out.0);

        let actions: Vec<Action> = vec![Action::Swap(SwapAction {
            pool_id: self.pool_id,
            token_in: buy_token.clone(),
            amount_in: Some(amount_to_proceed),
            token_out: sell_token.clone(),
            min_amount_out,
        })];

        let action = TokenReceiverMessage::Execute {
            force: true,
            actions,
        };

        let order = Order {
            status: OrderStatus::Pending,
            order_type,
            amount: Balance::from(amount_to_proceed),
            sell_token: sell_token.clone(),
            buy_token: buy_token.clone(),
            leverage: BigDecimal::from(leverage),
            sell_token_price: Price {
                ticker_id: self.view_price(sell_token.clone()).ticker_id,
                value: self.view_price(sell_token.clone()).value,
            },
            buy_token_price: Price {
                ticker_id: self.view_price(buy_token.clone()).ticker_id,
                value: self.view_price(buy_token.clone()).value,
            },
            block: env::block_height(),
        };

        ext_token::ext(sell_token.clone())
            .with_static_gas(Gas(3))
            .with_attached_deposit(1)
            .ft_transfer_call(
                REF_FINANCE.parse().unwrap(),
                amount,
                Some("Deposit tokens".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(20))
                    .with_attached_deposit(NO_DEPOSIT)
                    .swap_callback(user, amount, order),
            )
            .into()
    }

    #[private]
    pub fn swap_callback(
        &mut self,
        user: AccountId,
        amount: WBalance,
        order: Order,
    ) -> PromiseOrValue<WBalance> {
        require!(is_promise_success(), "Token swap hasn't end successfully");

        self.decrease_balance(user.clone(), order.sell_token.clone(), amount.0);

        ///// If user has a LPT with same pool_id&pl&pr,
        // /// it is an increase opertaion, else mint.
        // /// cause there is a UnorederMap<pool_id:lp:rp, lptid>; per user.
        // /// @param pool_id: a string like token_a|token_b|fee
        // /// @param left_point: left point of this range
        // /// @param right_point: right point of this range
        // /// @param amount_x: the number of token X users expect to add liquidity to use
        // /// @param amount_y: the number of token Y users expect to add liquidity to use
        // /// @param min_amount_x: the minimum number of token X users expect to add liquidity to use
        // /// @param min_amount_y: the minimum number of token Y users expect to add liquidity to use
        // /// @return the exist or new-mint lp token id, a string like pool_id|inner_id
        // pub fn add_liquidity(
        //     &mut self,
        //     pool_id: PoolId,
        //     left_point: i32,
        //     right_point: i32,
        //     amount_x: U128,
        //     amount_y: U128,
        //     min_amount_x: U128,
        //     min_amount_y: U128,
        // ) ➝ LptId

        // TODO add ref finance call to add concentrated_liquidity

        self.add_order(user, order);

        PromiseOrValue::Value(0.into())
    }

    #[private]
    pub fn set_pool_id(&mut self, pool_id: U128) {
        self.pool_id = pool_id.0 as u64;
    }

    #[private]
    pub fn add_order(&mut self, account_id: AccountId, order: Order) {}

    pub fn borrow_buy_token(&self, amount: U128, token: AccountId) {
        require!(
            env::prepaid_gas() >= GAS_FOR_BORROW,
            "Prepaid gas is not enough for borrow flow"
        );

        assert!(
            Balance::from(amount) > 0,
            "Amount should be a positive number"
        );

        let token_market = self.get_market_by_token(token).unwrap();

        ext_market::ext(token_market)
            .with_static_gas(GAS_FOR_BORROW)
            .with_attached_deposit(NO_DEPOSIT)
            .borrow(amount);
    }
}
