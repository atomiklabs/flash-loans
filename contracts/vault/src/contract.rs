#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    SubMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-flash-loan-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        ExecuteMsg::LendAsset { asset, borrower_addr } => {
            assert_eq!(is_whitelisted_borrower_gateway(&info.sender), true);
            assert_eq!(is_requested_asset_available(&asset), true);

            execute_lend_asset(deps, env, info, asset, borrower_addr)
        }
    }
}

fn execute_lend_asset(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    asset: Coin,
    borrower_addr: String,
) -> Result<Response, ContractError> {
    let borrower_addr = deps.api.addr_validate(borrower_addr.as_str())?;
    println!("[Vault: execute_lend_asset] asset = {:?} | recepient = {:?}", &asset, &info.sender);
    let msgs = vec![SubMsg::new(BankMsg::Send {
        to_address: borrower_addr.into(),
        amount: vec![asset],
    })];

    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("method", "lend"))
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
        let msg = ExecuteMsg::LendAsset {
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
        assert_eq!(1, response.messages.len());
        assert_eq!(("method", "lend"), response.attributes[0]);
        assert_eq!(
            response.messages[0],
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: borrower,
                amount: vec![asset_to_borrow],
            }))
        );
    }
}
