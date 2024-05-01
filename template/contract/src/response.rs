use cosmwasm_std::{StdError, Response};
use thiserror::Error;

pub type ContractResponse = Result<Response, ContractError>;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
}
