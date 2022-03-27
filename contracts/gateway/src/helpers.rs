use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, BankMsg, Coin, CosmosMsg, StdResult, WasmMsg, QuerierWrapper};

use crate::msg::{ExecuteMsg, QueryMsg};

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
        request_flash_loan_props: RequestFlashLoanProps<Msg>,
    ) -> StdResult<CosmosMsg> {
        let request_flash_loan_msg = ExecuteMsg::RequestFlashLoan {
            asset: request_flash_loan_props.asset,
            on_funded_msg: to_binary(&request_flash_loan_props.on_flash_loan_provided_hook)?,
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

    /// Tells how much the borrower still owes to the gateway
    pub fn get_debt_remaining(&self, querier: &QuerierWrapper, borrower: Addr) -> StdResult<Coin> {
        let (total_repayment, _): (Coin, Coin) = querier.query_wasm_smart(
            self.addr(),
            &QueryMsg::DebtRemaining { borrower },
        )?;

        Ok(total_repayment)
    }
}

pub struct RequestFlashLoanProps<'a, Msg: Serialize> {
    /// The asset to be borrowed
    pub asset: Coin,
    /// The message to be called back when flash-borrowed funds are available
    pub on_flash_loan_provided_hook: &'a Msg,
}

pub struct PayFlashLoanBackProps {
    /// The asset to be returned
    pub asset: Coin,
}
pub struct GetDebtRemainingProps<'a, Msg: Serialize> {
    /// The asset to be borrowed
    pub asset: Coin,
    /// The message to be called back when flash-borrowed funds are available
    pub on_flash_loan_provided_hook: &'a Msg,
}
