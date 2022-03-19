use crate::msg::StateResponse;
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
    has_coins, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

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

    let cw_gateway_contract_addr = deps.api.addr_validate(&msg.cw_gateway_contract_addr)?;

    STATE.save(
        deps.storage,
        &State {
            cw_gateway_contract_addr,
            reentrancy_prevention_flag: msg.reentrancy_prevention_flag,
            minimal_transfer_requirement: msg.minimal_transfer_requirement,
            reply_on_mode: msg.reply_on_mode,
        },
    )?;

    Ok(Response::new().add_attribute("method", "instantiate"))
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

    let state = STATE.update(deps.storage, |mut state: State| -> StdResult<_> {
        state.reentrancy_prevention_flag = 1;

        Ok(state)
    })?;

    let submsg = SubMsg {
        id: 1,
        msg: lock_funds_msg,
        reply_on: state.reply_on_mode,
        gas_limit: None,
    };

    Ok(Response::new().add_submessage(submsg))
}

fn lock_funds(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let State {
        minimal_transfer_requirement,
        ..
    } = STATE.load(deps.storage)?;

    if let Some(minimal_transfer) = minimal_transfer_requirement {
        if !has_coins(&info.funds, &minimal_transfer) {
            return Err(ContractError::TransferAmountLowerThanRequired {
                amount: minimal_transfer.amount.u128(),
                denom: minimal_transfer.denom,
            });
        }
    }

    Ok(Response::new().add_attribute("funds.locked", "true"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // let reentrancy_prevention_flag = match msg.result {
    //     SubMsgResult::Ok(_) => 2,
    //     SubMsgResult::Err(_) => 3,
    // };

    let state = STATE.update(deps.storage, |mut state: State| -> StdResult<_> {
        state.reentrancy_prevention_flag = 2;

        Ok(state)
    })?;

    let gateway_state: cw_gateway::msg::StateResponse = deps.querier.query_wasm_smart(
        state.cw_gateway_contract_addr.clone(),
        &cw_gateway::msg::QueryMsg::State,
    )?;

    let gateway_broadcast_fee = match gateway_state.state.broadcast_fee {
        Some(required_fee) => vec![required_fee],
        _ => vec![],
    };

    let broadcast_transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.cw_gateway_contract_addr.to_string(),
        funds: gateway_broadcast_fee,
        msg: to_binary(&cw_gateway::msg::ExecuteMsg::BroadcastTransfer {})?,
    });
    Ok(Response::new().add_message(broadcast_transfer_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(to_binary(&match msg {
        QueryMsg::State => query_state(deps)?,
    })?)
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(StateResponse { state })
}
