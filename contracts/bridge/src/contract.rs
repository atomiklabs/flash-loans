///
/// This is a minimal example enabling the following interaction:
/// 
/// 1. [Contract A: InitiateTransfer] A user initiates a transfer of their assets.
/// 2. [Contract B: LockFunds] The funds need to be locked within the contract.
///    Here it's just dummy lock of already sent coins, but the real example covers also CW20, and CW721.
///    That's why I created a call to another smart contract.
/// 3. [Contract A: reply] A reply to the LockFunds submessage needs to call another contract.
/// 4. [Contract C: BroadcastTransfer] This is an example of the another contract, which charges a fee in native coins
/// 
/// Question:
/// How can I access `info.funds` within the `reply` handler (step 3.), so I could include the fee while calling broadcast transfer handler (step 4.)?
///
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, has_coins, to_binary, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg, WasmMsg, Binary, Deps,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::STATE;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-bridge";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let _ = STATE.update(deps.storage, |state| -> StdResult<_> {
        let mut state = state;

        state.cw_gateway_contract_addr = msg.cw_gateway_contract_addr;

        Ok(state)
    });

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
    }
}

fn minimal_transfer_requirement() -> Coin {
    coin(10_000, "uluna") // 0.01 LUNA
}

/// Allows a user to start a trasaction.
/// It locks users native coin, and emits an event with the transaction details.
/// Some 3rd party system observes the broadcast method response
/// and picks up transfer data to complete the transfer on another blockchain.
fn initiate_transfer(
    _deps: DepsMut,
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

fn lock_funds(_deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let minimal_transfer = minimal_transfer_requirement();

    if !has_coins(&info.funds, &minimal_transfer) {
        return Err(ContractError::TransferAmountLowerThanRequired {
            amount: minimal_transfer.amount.u128(),
            denom: minimal_transfer.denom,
        });
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, env: Env, _msg: Reply) -> StdResult<Response> {
    let broadcast_transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds: vec![],
        msg: to_binary(&cw_gateway::msg::ExecuteMsg::BroadcastTransfer {})?,
    });
    Ok(Response::new().add_message(broadcast_transfer_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    to_binary(&vec![1])
}