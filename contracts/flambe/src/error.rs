use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Slippage Error")]
    SlippageError {},

    #[error("Insufficient Funds")]
    InsufficientFunds {},

    #[error("Pump Closed")]
    PumpClosed {},

    #[error("Status not in Pending")]
    NotPending {},

    #[error("Pump Open")]
    PumpOpen {},

    #[error("Invalid Fee")]
    InvalidFee {},
}
