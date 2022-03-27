use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, BankMsg, Coin, CosmosMsg, StdResult, WasmMsg};

use crate::msg::ExecuteMsg;

/// CwBridgeContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Contract(pub Addr);

impl Contract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(
        &self,
        msg: T,
        funds: Option<Vec<Coin>>,
    ) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: funds.unwrap_or(vec![]),
        }
        .into())
    }

    /// Creates a message calling the gatway to request a new flash loan
    pub fn request_flash_loan<Msg: Serialize>(
        &self,
        asset: Coin,
        on_flash_loan_provided_hook: &Msg,
    ) -> StdResult<CosmosMsg> {
        let request_flash_loan_msg = ExecuteMsg::RequestFlashLoan {
            on_funded_msg: to_binary(&on_flash_loan_provided_hook)?,
            asset,
        };

        self.call(request_flash_loan_msg, None)
    }

    /// Creates a message calling the repayment to pay back the current new flash loan
    pub fn repay_flash_loan(&self, asset: Coin) -> StdResult<CosmosMsg> {
        Ok(BankMsg::Send {
            to_address: self.addr().into(),
            amount: vec![asset],
        }
        .into())
    }
}
