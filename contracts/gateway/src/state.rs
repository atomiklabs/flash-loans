use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Binary, Coin};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub cw_vault_contract_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TempState {
    pub borrower_contract_addr: Addr,
    pub borrower_requested_asset: Coin,
    pub on_funded_msg: Binary,
}

pub const STATE: Item<State> = Item::new("state");

pub const TEMP_STATE: Item<TempState> = Item::new("temp_state");
