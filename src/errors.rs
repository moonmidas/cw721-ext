use cosmwasm_std::StdError;
use cw721_base::ContractError as CW721ContractError;
use thiserror::Error;

/// This overrides the ContractError enum defined in cw721-base
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Token already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Token not found")]
    TokenNotFound {},

    #[error("Cannot update frozen token")]
    TokenFrozen {},

    #[error("Token supply exhausted")]
    MaxTokenSupply {},

    #[error("failed")]
    Failed {},

    #[error("Maximum number of tokens have minted")]
    MaxTokensMinted {},

    #[error("This wallet has exceeded its mint limit")]
    MaxMintsPerAddress {},

    #[error("This wallet can't mint because it's not whitelisted")]
    NotWhitelisted {},

}

impl From<CW721ContractError> for ContractError {
    fn from(msg: CW721ContractError) -> ContractError {
        match msg {
            CW721ContractError::Unauthorized {} => ContractError::Unauthorized {},
            CW721ContractError::Claimed {} => ContractError::Claimed {},
            CW721ContractError::Expired {} => ContractError::Expired {},
            CW721ContractError::Std(e) => ContractError::Std(e),
        }
    }
}