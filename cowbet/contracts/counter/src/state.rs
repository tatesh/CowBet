
use cosmwasm_std::{ Addr, Uint128, Storage, Uint64,}; //u64?
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
//use percentage::Percentage;
use cw_storage_plus::Item;
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

static CONFIG_KEY: &[u8] = b"config";
static VAULT_KEY: &[u8] = b"vault";
static BANK_KEY: &[u8] = b"bank";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub denom: String,
    pub owner: Addr,
    pub vault_count: u64,
    pub bet_tokens: Uint128,
}

pub const STATE: Item<State> = Item::new("state");




#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenManager {
    pub token_balance: Uint128,             // total staked balance
    pub bet_token: Vec<(u64, Uint128)>,     //maps vault_id to 'weight voted'
    pub participated_vaults: Vec<u64>,       // vault_id
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Voter {
    pub vote: String, //yes or no
    pub weight: Uint128,
    pub pool_pct: f64, //how much you vault relative to everyone 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum VaultStatus {
    Closed,
    DepositsOpen, 
    InProgress,
    Finished,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Vault {
    pub creator: Addr,
    pub status: VaultStatus,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub sum_votes: Uint128,
    pub voters: Vec<Addr>,
    pub voter_info: Vec<Voter>,
    pub end_height: u64,
    pub start_height: Option<u64>,
    pub description: String,
    pub farmRewards: Uint128,
    pub result: String,
}


pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn vault(storage: &mut dyn Storage) -> Bucket<Vault> {
    bucket(storage, VAULT_KEY)
}

pub fn vault_read(storage: &dyn Storage) -> ReadonlyBucket<Vault> {
    bucket_read(storage, VAULT_KEY)
}

pub fn bank(storage: &mut dyn Storage) -> Bucket<TokenManager> {
    bucket(storage, BANK_KEY)
}

pub fn bank_read(storage: &dyn Storage) -> ReadonlyBucket<TokenManager> {
    bucket_read(storage, BANK_KEY)
}