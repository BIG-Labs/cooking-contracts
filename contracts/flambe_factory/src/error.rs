use cosmwasm_std::{Coin, StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Amount doesn't match funds")]
    WrongAmount {},

    #[error("Invalid Flambè Denom")]
    InvalidFlambeDenom {},

    #[error("Invalid Flambè Status")]
    InvalidFlambeStatus {},

    #[error("Invalid Flambe Setting Index: {index}")]
    InvalidFlambeSettingIndex { index: u8 },

    #[error("Invalid Empty Update")]
    InvalidEmptyUpdate,

    #[error("Insufficient Fee - requested {0} ")]
    InsufficientFee(Coin),

    #[error("Invalid Denom Registration Amount - requested {requested}")]
    InvalidDenomRegistrationAmount { requested: Uint128 },

    #[error("Invalid Reply ID: {0}")]
    InvalidReplyId(u64),
}
