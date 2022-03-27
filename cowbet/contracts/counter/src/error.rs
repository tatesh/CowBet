use cosmwasm_std::{OverflowError, StdError, Uint128, Uint64,};
use thiserror::Error;



#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("insufficient funds sent")]
    InsufficientFundsSent {},

    #[error("excessive withdrawal amount (max_amount {max_amount})")]
    ExcessiveWithdraw { max_amount: Uint128 },

    #[error("description too short (minimum description length {min_desc_length})")]
    DescriptionTooShort { min_desc_length: u64 },

    #[error("description too long (maximum description length {max_desc_length})")]
    DescriptionTooLong { max_desc_length: u64 },

    #[error("no bet in vault")]
    VaultNoBet {},

    #[error("Vault do not exist")]
    VaultDoesNotExist {},

    #[error("Poll cannot end in past")]
    VaultCannotEndInPast {},

    #[error("sender is not the creator of the Vault (sender {sender} creator {creator})")]
    VaultNotCreator { sender: String, creator: String },

    #[error("Vault is not in progress")]
    VaultNotInProgress {},

    #[error("Vault deposit period has not started (start_height {start_height})")]
    VaultDepositPeriodNotStarted { start_height: u64 },

    #[error("Vault deposit period has not expired (expire_height {expire_height})")]
    VaultDepositPeriodNotExpired { expire_height: u64 },

    #[error("sender has already voted in Vault")]
    VaultSenderVoted {},

    #[error("Vault is not open for deposits")]
    VaultNotOpenForDeposits {},

    #[error("Vault is not open for withdrawals")]
    VaultNotOpenForWithdrawals {},

    #[error("Invalid bet event")]
    InvalidBetEvent {},

    #[error("sender staked tokens insufficient")]
    VaultInsufficientStake {},

    #[error("sender amount tokens insufficient")]
    VaultInsufficientAmt {},

    #[error("Unauthorized action")]
    Unauthorized {},

}