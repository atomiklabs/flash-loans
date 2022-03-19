use cosmwasm_std::{Coin, ReplyOn};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub cw_gateway_contract_addr: String,
    pub reentrancy_prevention_flag: u8,
    pub minimal_transfer_requirement: Option<Coin>,
    pub reply_on_mode: ReplyOn,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    InitiateTransfer {},
    LockFunds {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    State,
}
