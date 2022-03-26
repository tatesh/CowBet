#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cosmwasm_std::{
    attr, coin, entry_point, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Storage, Uint128,
};
use cw2::set_contract_version;

use crate::coin_helpers::validate_sent_sufficient_coin;
use crate::error::ContractError;
use crate::msg::{CreateVaultResponse, ExecuteMsg, InstantiateMsg, VaultResponse, QueryMsg, TokenBetResponse};
use crate::state::{ bank, bank_read, config, config_read, vault, vault_read, Vault, VaultStatus, State, Voter,
};

const MIN_BET_AMOUNT: u128 = 1;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cowbet-v2";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        denom: msg.denom,
        owner: info.sender.clone(),
        vault_count: 0,
        bet_tokens: Uint128::zero(),
    };
    config(deps.storage).save(&state)?;

    Ok(Response::default())
    /*
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))*/
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BetTokens {} => bet_tokens(deps, env, info),
        ExecuteMsg::WithdrawRewards { amount } => {
            withdraw_bet_rewards(deps, env, info, amount)
        }
        ExecuteMsg::CastBet {
            vault_id,
            vote,
            weight,
        } => cast_bet(deps, env, info, vault_id, vote, weight),
        ExecuteMsg::EndDeposits { vault_id } => end_deposits(deps, env, info),
        ExecuteMsg::EndVault { vault_id } => end_vault(deps, env, info),
        ExecuteMsg::CreateVault {
            description,
            start_height,
            end_height,
        } => create_vault(
            deps,
            env,
            info,
            description,
            start_height,
            end_height,
        ),
        /*ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::Reset { count } => try_reset(deps, info, count),*/
    }
}

pub fn bet_tokens(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let key = info.sender.as_str().as_bytes();
    let mut token_manager = bank_read(deps.storage).may_load(key)?.unwrap_or_default();
    let mut state = config(deps.storage).load()?;
    validate_sent_sufficient_coin(&info.funds, Some(coin(MIN_BET_AMOUNT, &state.denom)))?;
    let funds = info
        .funds
        .iter()
        .find(|coin| coin.denom.eq(&state.denom))
        .unwrap();

    token_manager.token_balance += funds.amount;

    let bet_tokens = state.bet_tokens.u128() + funds.amount.u128();
    state.bet_tokens = Uint128::from(bet_tokens);
    config(deps.storage).save(&state)?;

    bank(deps.storage).save(key, &token_manager)?;

    Ok(Response::default())
}

pub fn withdraw_bet_rewards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let sender_address_raw = info.sender.as_str().as_bytes();
    /*
        Must add the farmed rewards + BET PROFIT
    */
    if let Some(mut token_manager) = bank_read(deps.storage).may_load(sender_address_raw)? {
        let largest_bet = bet_amount(&sender_address_raw, deps.storage);
        let withdraw_amount = amount.unwrap_or(token_manager.token_balance);
        if largest_bet + withdraw_amount > token_manager.token_balance {
            let max_amount = token_manager.token_balance.checked_sub(largest_bet)?;
            Err(ContractError::ExcessiveWithdraw { max_amount })
        } else {
            let balance = token_manager.token_balance.checked_sub(withdraw_amount)?;
            token_manager.token_balance = balance;

            bank(deps.storage).save(sender_address_raw, &token_manager)?;
            let mut state = config(deps.storage).load()?;
            let bet_tokens = state.bet_tokens.checked_sub(withdraw_amount)?;
            state.bet_tokens = bet_tokens;
            config(deps.storage).save(&state)?;

            Ok(send_tokens (
                &info.sender,
                vec![coin(withdraw_amount.u128(), &state.denom)],
                "approve",
            ))
        }
    } else {
        Err(ContractError::VaultNoBet {})
    }

}

/// validate_description returns an error if the description is invalid
fn validate_description(description: &str) -> Result<(), ContractError> {
    if (description.len() as u64) < MIN_DESC_LENGTH {
        Err(ContractError::DescriptionTooShort {
            min_desc_length: MIN_DESC_LENGTH,
        })
    } else if (description.len() as u64) > MAX_DESC_LENGTH {
        Err(ContractError::DescriptionTooLong {
            max_desc_length: MAX_DESC_LENGTH,
        })
    } else {
        Ok(())
    }
}
/// validate_end_height returns an error if the poll ends in the past
fn validate_end_height(end_height: Option<u64>, env: Env) -> Result<(), ContractError> {
    if end_height.is_some() && env.block.height >= end_height.unwrap() {
        Err(ContractError::VaultCannotEndInPast {})
    } else {
        Ok(())
    }
}

// create a new betting vault
pub fn create_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    description: String,
    start_height: Option<u64>,
    end_height: Option<u64>,
) -> Result<Response, ContractError> {
    validate_end_height(end_height, env.clone())?;
    validate_description(&description)?;

    let mut state = config(deps.storage).load()?;
    let vault_count = state.vault_count;
    let vault_id = state.vault_id + 1;
    state.vault_count = vault_id;

    let new_vault = Vault {
        creator: info.sender,
        status: VaultStatus::DepositsOpen, // Default to the 'Deposit' phase on creation.
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        voters: vec![],
        voter_info: vec![],
        end_height: end_height.unwrap_or(env.block.height + DEFAUL_END_HEIGHT_BLOCKS),
        start_height,
        description,
    };
    let key = state.vault_count.to_be_bytes();
    vault(deps.storage).save(&state)?;

    let r = Response{
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "create_vault"),
            attr("creator", new_vault.creator),
            attr("vault_id", &vault_id),
            attr("end_height", new_vault.end_height),
            attr("start_height", start_height.unwrap_or(0)),
        ],
        data: Some(to_binary(&CreateVaultResponse { vault_id })?),
    };
    Ok(r)
}

pub fn end_deposits(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault_id: u64,
) -> Result<Response, ContractError> {
    let key = &vault_id.to_be_bytes();
    let mut a_vault = vault(deps.storage).load(key)?;
    if a_vault.creator != info.sender {
        return Err(ContractError::VaultNotCreator {
            creator: a_vault.creator.to_string(),
            sender: info.sender.to_string(),
        });
    }
    if a_vault.status != VaultStatus::DepositsOpen {
        return Err(ContractError::VaultNotOpenForDeposits {});
    }
    if let Some(start_height) = a_vault.start_height {
        if start_height > env.block.height {
            return Err(ContractError::VaultDepositPeriodNotStarted { start_height });
        }
    }
    // Hmm?
    if a_vault.end_height > env.block.height {
        return Err(ContractError::VaultDepositPeriodNotExpired {
            expire_height: a_vault.end_height,
        });
    }

    let mut no = 0u128;
    let mut yes = 0u128;

    for voter in &a_vault.voter_info {
        if voter.vote == "yes" {
            yes += voter.weight.u128();
        } else {
            no += voter.weight.u128();
        }
    }
    let tallied_weight = yes + no;

    if env.block.height >= a_vault.end_height {
        for voter in &a_vault.voter_info {
            allow_claim(deps.storage, voter, vault_id)?;
        }
    }
    Ok(r)
}

// At the end of the epoch, allow the address to withdraw their rewards.
fn allow_claim(
    storage: &mut dyn Storage,
    voter: &Addr,
    vault_id: u64,
) -> Result<Response, ContractError> {
    let voter_key = &voter.as_str().as_bytes();
    let mut token_manager = bank_read(storage).load(voter_key).unwrap();
    // allow_claim entails of removing the mapped vault_id, & retaining the rest
    token_manager.bet_tokens.retain(|(k, _)| k != &vault_id);
    bank(storage).save(voter_key, &token_manager)?;
    Ok(Response::default())
}

fn bet_amount(voter: &[u8], storage: &dyn Storage) -> Uint128 {
    let token_manager = bank_read(storage).load(voter).unwrap();
    token_manager
        .bet_amount
        .iter()
        .map(|(_, v)| *v)
        .max()
        .unwrap_or_default()
}

pub fn cast_bet(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault_id: u64,
    vote: String,
    weight: Uint128,
) -> Result<Response, ContractError> {
    let vault_key = &vault_id.to_be_bytes();
    let state = config_read(deps.storage).load()?;
    if vault_id == 0 || state.vault_count > vault_id {
        return Err(ContractError::VaultDoesNotExist {});
    }
    if (state.vote != "Yes" && state.vote != "No") {
        return Err(ContractError::InvalidBetEvent {});
    }

    let mut a_vault - vault(deps.storage).load(vault_key)?;

    if a_vault.status != VaultStatus::DepositsOpen {
        return Err(ContractError::VaultNotOpenForDeposits {});
    }

    let key = info.sender.as_str().as_bytes();
    let mut token_manager = bank_read(deps.storage).may_load(key)?.unwrap_or_default();

    if token_manager.token_balance < weight {
        return Err(ContractError::VaultInsufficientAmt {});
    }
    token_manager.participated_vaults.push(vault_id);
    token_manager.bet_tokens.push((vault_id, weight));
    bank(deps.storage).save(key, &token_manager)?;

    a_vault.voters.push(info.sender.clone());

    // Need to include pool_pct for the voter.
    let voter_info = Voter {vote, weight}
    a_vault.voter_info.push(voter_info);
    vault(deps.storage).save(vault_key, &a_vault)?;

    let attributes = vec![
        attr("action", "vote_casted"),
        attr("vault_id", &vault_id),
        attr("weight", &weight),
        attr("voter", &info.sender),
    ];

    let r = Response {
        submessages: vec![],
        messages: vec![],
        attributes,
        data: None,
    };
    Ok(r)

}

fn send_tokens(to_address: &Addr, amount: Vec<Coin>, action: &str) -> Response {
    let attributes = vec![attr("action", action), attr("to", to_address.clone())];

    Response {
        submessages: vec![],
        messages: vec![CosmosMsg::Bank(BankMsg::Send {
            to_address: to_address.to_string(),
            amount,
        })],
        attributes,
        data: None,
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&config_read(deps.storage).load()?),
        QueryMsg::TokenBet { address } => {
            token_balance(deps, deps.api.addr_validate(address.as_str())?)
        }
        QueryMsg::Vault { vault_id } => query_vault(deps, vault_id),
    }
}

fn query_vault(deps: Deps, vault_id: u64) -> StdResult<Binary> {
    let key = &vault_id.to_be_bytes();

    let vault = match vault_read(deps.storage).may_load(key)? {
        Some(vault) => Some(vault),
        None => return Err(StdError::generic_err("Vault does not exist")),
    }
    .unwrap();

    let resp = VaultResponse {
        creator: vault.creator.to_string(),
        status: vault.status,
        end_height: Some(vault.end_height),
        start_height: vault.start_height,
        description: vault.description,
    };
    to_binary(&resp)
}

fn token_balance(deps: Deps, address: Addr) -> StdResult<Binary> {
    let token_manager = bank_read(deps.storage)
        .may_load(address.as_str().as_bytes())?
        .unwrap_or_default();

    let resp = TokenBetResponse {
        token_balance: token_manager.token_balance,
    };

    to_binary(&resp)
}

/*pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}*/
/*
pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
*/