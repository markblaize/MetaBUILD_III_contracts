use crate::*;

pub type WBalance = U128;
pub type Digits = u32;
pub const FEE_DIVISOR: u32 = 10_000;
pub type Digits = u32;

pub const MARKET_PLATFORM_ACCOUNT: &str = "omomo.nearlend.testnet";

#[ext_contract(ext_token)]
trait NEP141Token {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: WBalance,
        memo: Option<String>,
        msg: String,
    );
}

#[near_bindgen]
impl Contract {
    pub fn set_price(&mut self, market_id: AccountId, price: Price) {
        self.prices.insert(&market_id, &price);
    }
}

impl Contract {
    pub fn get_price_by_token(&self, token_id: AccountId) -> WBalance {
        assert!(
            self.prices.get(&token_id).is_some(),
            "There no such prices set yet"
        );
        self.prices.get(&token_id).unwrap().value
    }

    pub fn calculate_xrate(&self, token_id_1: AccountId, token_id_2: AccountId) -> Ratio {
        Ratio::from(self.get_price_by_token(token_id_1))
            / Ratio::from(self.get_price_by_token(token_id_2))
    }

    pub fn is_valid_market_call(&self) -> bool {
        self.markets.contains(&env::predecessor_account_id())
    }
}
