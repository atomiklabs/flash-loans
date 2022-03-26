#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    WasmMsg, SubMsg, BankMsg, coin,
};
use cw2::set_contract_version;
use cw_multi_test::Wasm;

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

    println!("[Borrower: exectute_open_flash_loan]: trying to borrow {:?}", &asset_to_borrow);

    let on_fundend_msg = ExecuteMsg::OnFlashLoanProvided {};

    let request_flash_loan_msg = cw_flash_loan_gateway::msg::ExecuteMsg::RequestFlashLoan {
        asset: asset_to_borrow,
        on_funded_msg: to_binary(&on_fundend_msg)?,
    };

    let msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cw_gateway_contract_addr.to_string(),
        funds: vec![],
        msg: to_binary(&request_flash_loan_msg)?,
    })];

    Ok(Response::new().add_messages(msgs))
}

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
    let msgs = vec![SubMsg::new(BankMsg::Send {
        to_address: cw_gateway_contract_addr.into(),
        amount: vec![coin(175_000_000 - 1, "uluna")],
    })];

    Ok(Response::new()
    .add_submessages(msgs)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    let stub: [u8; 0] = [];
    to_binary(&stub)
}
