use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError{val: String},

    #[error("Not enough funds to cover fee")]
    NotEnoughFundsToCoverFee,

    #[error("Not enough funds to cover loan repayment")]
    NotEnoughFundsToCoverLoanRepayment,

    #[error("Requested asset was not provided by vault")]
    RequestedAssetNotProvided,

    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
