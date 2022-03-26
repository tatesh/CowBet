use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

static CONFIG_KEY: &[u8] = b"config";
static POLL_KEY: &[u8] = b"bets";
static BANK_KEY: &[u8] = b"bank";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
    pub bet_count: u64,
    pub staked_tokens: Uint128,
}

pub const STATE: Item<State> = Item::new("state");




#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenManager {
    pub token_balance: Uint128,             // total staked balance
    pub locked_tokens: Vec<(u64, Uint128)>, //maps poll_id to weight voted
    pub participated_bets: Vec<u64>,       // bet_id
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BetParticipant { //previously Voter from example
    pub vote: String, //yes or no
    pub weight: Uint128,
    pub pool_pct: Uint128, //how much you bet relative to everyone 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum BettingStatus {
    InProgress,
    Tally,
    Finished,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bet {
    pub creator: Addr,
    pub status: PollStatus,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub bet_participants: Vec<Addr>,
    pub bet_partic_info: Vec<Voter>,
    pub end_height: u64,
    pub start_height: Option<u64>,
    pub description: String,
    pub farmRewards: Uint128, 
}


pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn bet(storage: &mut dyn Storage) -> Bucket<Poll> {
    bucket(storage, BET_KEY)
}

pub fn bet_read(storage: &dyn Storage) -> ReadonlyBucket<Poll> {
    bucket_read(storage, BET_KEY)
}

pub fn bank(storage: &mut dyn Storage) -> Bucket<TokenManager> {
    bucket(storage, BANK_KEY)
}

pub fn bank_read(storage: &dyn Storage) -> ReadonlyBucket<TokenManager> {
    bucket_read(storage, BANK_KEY)
}