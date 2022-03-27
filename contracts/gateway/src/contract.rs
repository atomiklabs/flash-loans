use std::convert::TryFrom;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, FlashLoanState, CONFIG, FLASH_LOAN_STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-flash-loan-gateway";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cw_vault_contract_addr = deps.api.addr_validate(&msg.cw_vault_contract_addr)?;

    CONFIG.save(
        deps.storage,
        &Config {
            cw_vault_contract_addr,
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
        ExecuteMsg::RequestFlashLoan {
            asset,
            on_funded_msg,
        } => execute_request_flash_loan(deps, env, info, asset, on_funded_msg),
    }
}

fn execute_request_flash_loan(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    asset: Coin,
    on_funded_msg: Binary,
) -> Result<Response, ContractError> {
    let Config {
        cw_vault_contract_addr,
    } = CONFIG.load(deps.storage)?;

    println!(
        "[Gateway: execute_request_flash_loan]: asking vault to lend {:?}",
        &asset
    );

    let flash_loan_state = FlashLoanState {
        borrower_contract_addr: info.sender,
        borrower_requested_asset: asset.clone(),
        on_funded_msg,
    };

    FLASH_LOAN_STATE.save(deps.storage, &flash_loan_state)?;

    let lend_asset_msg = cw_flash_loan_vault::msg::ExecuteMsg::LendAsset {
        asset,
        borrower_addr: flash_loan_state.borrower_contract_addr.to_string(),
    };

    let msgs = vec![
        // firstly, request funds from vault
        SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: cw_vault_contract_addr.to_string(),
                funds: vec![],
                msg: to_binary(&lend_asset_msg)?,
            },
            ReplyToGateway::OnLendAssetCompleted as u64,
        ),
    ];

    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("method", "request_flash_loan"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.result.is_err() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: msg.result.unwrap_err(),
        }));
    }

    match ReplyToGateway::try_from(msg.id) {
        Ok(ReplyToGateway::OnLendAssetCompleted) => reply_on_lend_asset_completed(deps, env),
        Ok(ReplyToGateway::OnExternalHandlerCompleted) => {
            reply_on_external_handler_completed(deps, env)
        }
        _ => Err(ContractError::Std(StdError::GenericErr {
            msg: format!("reply id `{:?}` is invalid", msg.id),
        })),
    }
}

fn reply_on_lend_asset_completed(deps: DepsMut, _env: Env) -> Result<Response, ContractError> {
    let temp_state = FLASH_LOAN_STATE.load(deps.storage)?;

    let msgs = vec![
        // secondly, send funds from gateway to the borrower
        SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: temp_state.borrower_contract_addr.to_string(),
                funds: vec![],
                msg: temp_state.on_funded_msg,
            },
            ReplyToGateway::OnExternalHandlerCompleted as u64,
        ),
    ];

    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("method", "on_lend_assets_completed"))
}

fn reply_on_external_handler_completed(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let flash_loan_state = FLASH_LOAN_STATE.load(deps.storage)?;

    let repayment_amount_base = flash_loan_state.borrower_requested_asset;
    println!(
        "[Gateway: repay the loan to the vault]: base = {:?}",
        &repayment_amount_base
    );

    let (repayment_amount_total, repayment_amount_vault) = calculate_debt_remaining(
        repayment_amount_base.clone(),
        config.cw_vault_contract_addr.clone(),
    )?;

    println!(
        "[Gateway: repay the loan]: total = {:?}",
        &repayment_amount_total
    );
    println!(
        "[Gateway: repay the loan]: base with vault's fees = {:?}",
        &repayment_amount_vault
    );

    // FIXME: this is a naive way to check balance
    //        better allow the borrower to call a getway payback method that will accept funds directly
    let gateway_current_balance = deps.querier.query_balance(
        env.contract.address.to_string(),
        repayment_amount_base.denom.as_str(),
    )?;

    if gateway_current_balance.amount < repayment_amount_total.amount {
        return Err(ContractError::NotEnoughFundsToCoverFee);
    }

    let msgs = vec![SubMsg::new(BankMsg::Send {
        to_address: config.cw_vault_contract_addr.into(),
        // amount: vec![repayment_amount],
        // TODO: extend the amount base with the relevant fees
        amount: vec![repayment_amount_vault],
    })];

    FLASH_LOAN_STATE.remove(deps.storage);

    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("method", "on_external_handler_completed"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(to_binary(&match msg {
        QueryMsg::DebtRemaining { borrower } => query_debt_remaining(deps, borrower)?,
    })?)
}

fn query_debt_remaining(deps: Deps, borrower: Addr) -> StdResult<(Coin, Coin)> {
    let config = CONFIG.load(deps.storage)?;
    let flash_loan_state = FLASH_LOAN_STATE.load(deps.storage)?;

    if flash_loan_state.borrower_contract_addr != borrower {
        return Err(StdError::generic_err("Cannot retrieve debt remaining"));
    }

    Ok(calculate_debt_remaining(
        flash_loan_state.borrower_requested_asset,
        config.cw_vault_contract_addr,
    )?)
}

fn calculate_debt_remaining(
    repayment_amount_base: Coin,
    _vault_addr: Addr,
) -> StdResult<(Coin, Coin)> {
    let repayment_amount_base_gatway_fee = repayment_amount_base
        .amount
        // TODO: load config param to check current gateway
        .checked_div(Uint128::from(100u128))?;

    let repayment_amount_base_vault_fee = repayment_amount_base
        .amount
        // TODO: query vault to get its current fee
        .checked_div(Uint128::from(50u128))?;

    let repayment_amount_for_vault = repayment_amount_base
        .amount
        .checked_add(repayment_amount_base_vault_fee.clone())?;

    let repayment_total_value =
        repayment_amount_for_vault.checked_add(repayment_amount_base_gatway_fee)?;

    Ok((
        coin(
            repayment_total_value.u128(),
            repayment_amount_base.denom.as_str(),
        ),
        coin(
            repayment_amount_for_vault.u128(),
            repayment_amount_base.denom.as_str(),
        ),
    ))
}

enum ReplyToGateway {
    OnLendAssetCompleted = 1,
    OnExternalHandlerCompleted = 2,
}

impl TryFrom<u64> for ReplyToGateway {
    type Error = ();

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::OnLendAssetCompleted as u64 => Ok(Self::OnLendAssetCompleted),
            x if x == Self::OnExternalHandlerCompleted as u64 => {
                Ok(Self::OnExternalHandlerCompleted)
            }
            _ => Err(()),
        }
    }
}
