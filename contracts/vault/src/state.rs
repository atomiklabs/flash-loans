use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Coin;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub broadcast_fee: Option<Coin>,
}

pub const STATE: Item<State> = Item::new("state");

