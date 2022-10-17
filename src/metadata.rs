use crate::big_decimal::{BigDecimal, WBalance, WRatio};
use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{Balance, BlockHeight, BorshStorageKey};

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    Markets,
    Prices,
    Orders,
    SupportedMarkets,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MarketData {
    pub total_supplies: WBalance,
    pub total_borrows: WBalance,
    pub total_reserves: WBalance,
    pub exchange_rate_ratio: WRatio,
    pub interest_rate_ratio: WRatio,
    pub borrow_rate_ratio: WRatio,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PnLView {
    pub(crate) is_profit: bool,
    pub(crate) amount: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Price {
    ticker_id: String,
    value: BigDecimal,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum OrderStatus {
    Pending,
    Executed,
    Canceled,
    Liquidated,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Order {
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub amount: Balance,
    pub sell_token: AccountId,
    pub buy_token: AccountId,
    pub leverage: BigDecimal,
    pub sell_token_price: Price,
    pub buy_token_price: Price,
    pub block: BlockHeight,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OrderView {
    pub order_id: U128,
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub amount: Balance,
    pub sell_token: AccountId,
    pub buy_token: AccountId,
    pub buy_token_price: WBalance,
    pub fee: WBalance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TradePair {
    pub sell_ticker_id: String,
    pub sell_token: AccountId,
    pub sell_token_market: AccountId,
    pub buy_ticker_id: String,
    pub buy_token: AccountId,
}
