use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Binary, Coin};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub cw_vault_contract_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FlashLoanState {
    pub borrower_contract_addr: Addr,
    pub borrower_requested_asset: Coin,
    pub on_funded_msg: Binary,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const FLASH_LOAN_STATE: Item<FlashLoanState> = Item::new("flash_loan_state");
