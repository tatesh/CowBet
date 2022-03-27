use crate::state::VaultStatus;
use cosmwasm_std::{ Uint128, Uint64,};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CastBet {
        vault_id: u64,
        vote: String,
        weight: Uint128,
    },
    BetTokens {},
    WithdrawRewards {
        amount: Option<Uint128>,
    },
    CreateVault {
        description: String,
        start_height: Option<u64>,
        end_height: Option<u64>,
    },
    EndDeposits {
        vault_id: u64,
    },
    EndVault {
        vault_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    TokenBet { address: String },
    Vault { vault_id: u64 },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct VaultResponse {
    pub creator: String,
    pub status: VaultStatus,
    pub end_height: Option<u64>,
    pub start_height: Option<u64>,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct CreateVaultResponse {
    pub vault_id: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct VaultCountResponse {
    pub vault_count: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct TokenBetResponse {
    pub token_balance: Uint128,
}