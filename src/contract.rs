#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, has_coins, to_binary, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg, WasmMsg, Binary, Deps,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-template";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InitiateTransfer {} => initiate_transfer(deps, env, info),
        ExecuteMsg::LockFunds {} => lock_funds(deps, env, info),
        ExecuteMsg::BroadcastTransfer {} => broadcast_transfer(deps, env, info),
    }
}

fn transfer_fee() -> Coin {
    coin(990, "uluna")
}

fn minimal_transfer_requirement() -> Coin {
    coin(10_000, "uluna") // 0.01 LUNA
}

/// Allows a user to start a trasaction.
/// It locks users native coin, and emits an event with the transaction details.
/// Some 3rd party system observes the broadcast method response
/// and picks up transfer data to complete the transfer on another blockchain.
fn initiate_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let lock_funds_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds: info.funds,
        msg: to_binary(&ExecuteMsg::LockFunds {})?,
    });
    Ok(Response::new().add_submessage(SubMsg::reply_on_success(lock_funds_msg, 1)))
}

fn lock_funds(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let minimal_transfer = minimal_transfer_requirement();

    if !has_coins(&info.funds, &minimal_transfer) {
        return Err(ContractError::TransferAmountLowerThanRequired {
            amount: minimal_transfer.amount.u128(),
            denom: minimal_transfer.denom,
        });
    }

    Ok(Response::default())
}

fn broadcast_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if !has_coins(&info.funds, &transfer_fee()) {
        return Err(ContractError::NotEnoughFundsToCoverFee);
    }

    Ok(Response::new().add_attribute("transfer.status", "initiated"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, _msg: Reply) -> StdResult<Response> {
    let broadcast_transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::BroadcastTransfer {})?,
    });
    Ok(Response::new().add_message(broadcast_transfer_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    to_binary(&vec![1])
}