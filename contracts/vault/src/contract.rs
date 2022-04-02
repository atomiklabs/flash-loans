#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, wasm_execute, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{BroadcastMsg, ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-flash-loan-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const REPLY_ON_ASSET_REPAYMENT: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
        ExecuteMsg::ProvideAsset {
            asset,
            borrower_addr,
        } => {
            if !is_whitelisted_borrower_gateway(&info.sender) {
                return Err(ContractError::Unauthorized {});
            }

            if !is_requested_asset_available(&asset) {
                return Err(ContractError::AssetUnavailable {});
            }

            execute_provide_asset(deps, env, info, asset, borrower_addr)
        }
    }
}

fn execute_provide_asset(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    asset: Coin,
    borrower_addr: String,
) -> Result<Response, ContractError> {
    let borrower_addr = deps.api.addr_validate(borrower_addr.as_str())?;

    println!(
        "[Vault: execute_provide_asset] asset = {:?} | recepient = {:?}",
        &asset, &borrower_addr
    );

    let msgs = vec![
        // let's have the flash loan sent to the borrower
        SubMsg::new(BankMsg::Send {
            to_address: borrower_addr.clone().into(),
            amount: vec![asset.clone()],
        }),
        SubMsg::reply_on_success(
            wasm_execute(
                info.sender,
                // call the sender (the vault) back to let it know the funds were sent to the borrower
                // TODO: extract the message into a package so it could be shared between the gateway and the vault
                &BroadcastMsg::FlashLoanProvided {
                    asset,
                    borrower_addr: borrower_addr.into(),
                },
                vec![],
            )?,
            REPLY_ON_ASSET_REPAYMENT,
        ),
    ];

    Ok(Response::new()
        .add_submessages(msgs)
        .add_attributes(vec![("module", "vault"), ("action", "execute_provide_asset")]))
}

/// Is provided address is on the borrower gatway whitelist?
fn is_whitelisted_borrower_gateway(_address: &Addr) -> bool {
    true
}

/// Is the requested asset available for lending?
fn is_requested_asset_available(_requested_asset: &Coin) -> bool {
    true
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.result.is_err() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: msg.result.unwrap_err(),
        }));
    }

    match msg.id {
        REPLY_ON_ASSET_REPAYMENT => reply_on_asset_repayment(deps, env),
        _ => Err(ContractError::Std(StdError::GenericErr {
            msg: format!("reply id `{:?}` is invalid", msg.id),
        })),
    }
}

fn reply_on_asset_repayment(_deps: DepsMut, _env: Env) -> Result<Response, ContractError> {
    // TODO: validate repayment
    println!("[Vault: reply_on_asset_repayment] TODO: validate repayment");
    // if this handler fails, the whole trasaction will be reverted

    Ok(Response::new().add_attributes(vec![
        ("module", "vault"),
        ("action", "reply_on_asset_repayment"),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    let stub: [u8; 0] = [];
    Ok(to_binary(&stub)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::{
        coin,
        testing::{mock_dependencies, mock_env, mock_info},
        BankMsg, CosmosMsg, SubMsg,
    };

    #[test]
    fn instantiate_vault() {
        let mut deps = mock_dependencies();

        let creator = String::from("creator");
        let msg = InstantiateMsg {};
        let info = mock_info(&creator, &[coin(1_000_000_000, "uluna")]);

        let result = instantiate(deps.as_mut(), mock_env(), info, msg);

        assert_eq!(
            result.is_ok(),
            true,
            "Initializes vault contract successfully"
        );
    }

    #[test]
    fn request_vault_to_lend_funds() {
        let mut deps = mock_dependencies();

        let creator = String::from("creator");
        let msg = InstantiateMsg {};
        let info = mock_info(&creator, &[coin(1_000_000_000, "uluna")]);

        let _ = instantiate(deps.as_mut(), mock_env(), info, msg);

        let borrower = String::from("borrower");
        let asset_to_borrow = coin(20_000_000, "uluna");
        let msg = ExecuteMsg::ProvideAsset {
            asset: asset_to_borrow.clone(),
            borrower_addr: borrower.clone(),
        };
        let info = mock_info(&borrower, &[]);

        let result = execute(deps.as_mut(), mock_env(), info, msg);

        assert_eq!(
            result.is_ok(),
            true,
            "Initializes vault contract successfully"
        );

        let response = result.unwrap();
        assert_eq!(2, response.messages.len());
        assert_eq!(("module", "vault"), response.attributes[0]);
        assert_eq!(("action", "execute_provide_asset"), response.attributes[1]);
        assert_eq!(
            response.messages[0],
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: borrower,
                amount: vec![asset_to_borrow],
            }))
        );
    }
}
