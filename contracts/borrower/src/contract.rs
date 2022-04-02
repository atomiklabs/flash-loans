#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use cw2::set_contract_version;
use cw_flash_loan_gateway::helpers::{Contract as FlashLoanGateway, RequestFlashLoanProps};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Config, CONFIG},
};

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

    CONFIG.save(
        deps.storage,
        &Config {
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

/// Handler initiating a flash loan
fn exectute_open_flash_loan(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    asset_to_borrow: Coin,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // This one tells the gateway about which message to execute while calling
    // the borrower contract back with the flash-loaned funds.
    let on_flash_loan_provided_hook = &ExecuteMsg::OnFlashLoanProvided {};

    let msgs = vec![
        FlashLoanGateway(config.cw_gateway_contract_addr).request_flash_loan(
            RequestFlashLoanProps {
                asset: asset_to_borrow,
                on_flash_loan_provided_hook,
            },
        )?,
    ];

    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        ("module", "borrower"),
        ("action", "execute_open_flash_loan"),
    ]))
}

/// Handler utilising the flash loan.
/// Allows executing any set of arbitrary messages with funds provided by the flash loan.
fn execute_on_flash_loan_provided(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Money's in — time for swaps.

    let available_luna = deps
        .querier
        .query_balance(env.contract.address.clone(), "uluna")?;
    println!(
        "[Borrower: execute_on_flash_loan_provided]: available uluna = {:?}",
        available_luna
    );

    let mut msgs: Vec<CosmosMsg> = vec![
        // TODO: add any arbitrary messages to perform required transactions
    ];

    // Repay the flash loan
    let flash_loan_gateway = FlashLoanGateway(config.cw_gateway_contract_addr);

    let total_repayment =
        flash_loan_gateway.get_debt_remaining(&deps.querier, env.contract.address)?;

    msgs.push(flash_loan_gateway.repay_flash_loan(total_repayment)?);

    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        ("module", "borrower"),
        ("action", "execute_on_flash_loan_provided"),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    let stub: [u8; 0] = [];
    to_binary(&stub)
}
