use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub cw_gateway_contract_addr: Addr,
}

pub const STATE: Item<State> = Item::new("state");

pub const BALANCES: Map<String, Uint128> = Map::new("balances");
