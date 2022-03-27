#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use cw_flash_loan_gateway::helpers::Contract as FlashLoanGateway;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-flash-loans-borrower";
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
        ExecuteMsg::OpenFlashLoan { asset_to_borrow } => {
            exectute_open_flash_loan(deps, env, info, asset_to_borrow)
        }
        ExecuteMsg::OnFlashLoanProvided {} => execute_on_flash_loan_provided(deps, env, info),
    }
}
fn exectute_open_flash_loan(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    asset_to_borrow: Coin,
) -> Result<Response, ContractError> {
    let State {
        cw_gateway_contract_addr,
    } = STATE.load(deps.storage)?;

    println!(
        "[Borrower: exectute_open_flash_loan]: trying to borrow {:?}",
        &asset_to_borrow
    );

    let msgs = vec![FlashLoanGateway(cw_gateway_contract_addr)
        .request_flash_loan(asset_to_borrow, &ExecuteMsg::OnFlashLoanProvided {})?];

    Ok(Response::new().add_messages(msgs))
}

/// Handler for executing any arbitrary message(s) with funds provided by the flash loan.
fn execute_on_flash_loan_provided(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let State {
        cw_gateway_contract_addr,
    } = STATE.load(deps.storage)?;

    let luna_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), "uluna")?;

    println!(
        "[Borrower: on_flash_loan_provided]: total uluna balance = {:?}",
        &luna_balance
    );

    let mut msgs: Vec<CosmosMsg> = vec![];

    // TODO: add any arbitrary messages to perform required transactions

    msgs.push(
        FlashLoanGateway(cw_gateway_contract_addr).repay_flash_loan(coin(175_000_000, "uluna"))?,
    );

    Ok(Response::new().add_messages(msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    let stub: [u8; 0] = [];
    to_binary(&stub)
}
