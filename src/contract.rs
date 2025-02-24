#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut,
    Env, MessageInfo, Order, Response, StdResult, Uint128, WasmMsg,
};

use crate::state::{
    Cooperative, Loan, LoanStatus, Member, Proposal, ProposalOutcome, ProposalType, RiskProfile,
    Vote, WhitelistedToken, COOPERATIVES, COOPERATIVES_PROPOSALS, MEMBERS, PRICES, PROPOSALS,
    TOKENS,
};

use cw2::set_contract_version;
use execute::{
    execute_borrow, execute_create_cooperative, execute_fund_cooperative, execute_proposal,
    execute_propose, execute_repay, execute_update_price, execute_vote, execute_withdraw_weight,
    withdraw_contribution_and_rewards,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ajor";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State {
        owner: info.sender.clone(),
        weight_token: "uatom".to_string(),
        total_corporatives: 0,
        total_pooled_funds: vec![],
        current_proposal_id: 0,
        current_whitelisted_token_id: 0,
        current_loan_id: 0,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateTokenPrice {
            token_addr,
            usd_price,
        } => execute_update_price(deps, env, info, token_addr, usd_price),
        ExecuteMsg::CreateCooperative {
            name,
            risk_profile,
            initial_members,
            initial_whitelisted_tokens,
        } => execute_create_cooperative(
            deps,
            name,
            risk_profile,
            initial_members,
            initial_whitelisted_tokens,
        ),
        ExecuteMsg::FundCooperative {
            cooperative_name,
            token,
            is_native,
            amount,
        } => execute_fund_cooperative(deps, env, info, cooperative_name, token, is_native, amount),
        ExecuteMsg::Borrow {
            cooperative_name,
            tokens_in,
            amount_in,
            token_out,
            min_amount_out,
        } => execute_borrow(
            deps,
            env,
            info,
            cooperative_name,
            tokens_in,
            amount_in,
            token_out,
            min_amount_out,
        ),
        ExecuteMsg::Repay {
            cooperative_name,
            token,
        } => execute_repay(deps, env, info, cooperative_name, token),
        ExecuteMsg::Propose {
            cooperative_name,
            proposal,
        } => execute_propose(deps, env, info, cooperative_name, proposal),
        ExecuteMsg::Vote {
            cooperative_name,
            proposal_id,
            weight,
            aye,
        } => execute_vote(deps, env, info, cooperative_name, proposal_id, weight, aye),
        ExecuteMsg::WithdrawWeight {
            cooperative_name,
            proposal_id,
        } => execute_withdraw_weight(deps, env, info, cooperative_name, proposal_id),
        ExecuteMsg::WithdrawContributionAndReward {
            cooperative_name,
            token,
        } => withdraw_contribution_and_rewards(deps, env, info, cooperative_name, token),
        ExecuteMsg::ExecuteProposal {
            cooperative_name,
            proposal_id,
        } => execute_proposal(deps, env, info, cooperative_name, proposal_id),
    }
}

pub mod execute {
    use cosmwasm_std::{StdError, Timestamp};
    use cw20::Cw20ExecuteMsg;

    use crate::state::{Price, REWARDS_POOLS};

    use super::*;

    pub fn execute_update_price(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_addr: Addr,
        usd_price: u128,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;

        // only owner can update price
        // TODO: use oracle
        assert!(
            state.owner == info.sender,
            "{}",
            ContractError::Unauthorized {}
        );

        // Get token id
        let token_id = TOKENS.load(deps.storage, token_addr.clone())?;

        // construct price
        let price = Price {
            latest_price_to_usd: usd_price.into(),
            last_updated_at: Timestamp::from_seconds(env.block.time.seconds()),
        };

        // Update storage
        PRICES.save(deps.storage, token_id, &price)?;

        Ok(Response::new()
            .add_attribute("action", "update_price")
            .add_attribute("denom", token_addr.as_str())
            .add_attribute("price", usd_price.to_string()))
    }

    pub fn execute_create_cooperative(
        deps: DepsMut,
        name: String,
        risk_profile: RiskProfile,
        initial_members: Vec<Member>, //max 20 initial members
        initial_whitelisted_tokens: Vec<WhitelistedToken>, // max 5 whitelisted tokens,
    ) -> Result<Response, ContractError> {
        let name = name.trim().to_lowercase();
        // Check if cooperative already exists
        if COOPERATIVES.has(deps.storage, name.clone()) {
            return Err(ContractError::CooperativeAlreadyExists {});
        }

        // Check no more than 20 initial members
        assert!(initial_members.len().le(&20));

        // Check no more than 5 whitelisted tokens.
        assert!(initial_whitelisted_tokens.len().le(&5));

        // Create new corporative
        let cooperative = Cooperative {
            name: name.clone(),
            total_funds: vec![],
            members: initial_members,
            risk_profile,
            whitelisted_tokens: initial_whitelisted_tokens.clone(),
        };

        /* -- Update storage --- */
        let mut state = STATE.load(deps.storage)?;

        // add whitelisted token to
        for token in initial_whitelisted_tokens {
            let current_token_idx = state.current_whitelisted_token_id + 1;

            if !token.is_native {
                TOKENS.save(
                    deps.storage,
                    token.contract_addr.clone().unwrap(),
                    &current_token_idx,
                )?;
            } else {
                TOKENS.save(deps.storage, Addr::unchecked("NATIVE"), &current_token_idx)?;
            }
        }

        // update total cooperative
        state.total_corporatives += 1;
        state.current_whitelisted_token_id += 1;

        COOPERATIVES.save(deps.storage, name.clone(), &cooperative)?;
        STATE.save(deps.storage, &state)?;

        Ok(Response::new()
            .add_attribute("action", "create_society")
            .add_attribute("name", name))
    }

    pub fn execute_fund_cooperative(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cooperative_name: String,
        token: String,
        is_native: bool,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let mut cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;
        let token_idx;
        let mut messages: Vec<CosmosMsg> = vec![];

        // Validate transfer
        if is_native {
            let coin = info
                .funds
                .iter()
                .find(|coin| coin.denom == token)
                .ok_or(ContractError::NoFunds {})?;

            if coin.amount != amount {
                return Err(ContractError::FundsMustMatchAmount {});
            }

            // Validate token is whitelisted
            token_idx = cooperative
                .whitelisted_tokens
                .iter()
                .position(|x| x.denom == coin.denom);
        } else {
            let validate_msg = validate_cw20(&deps.as_ref(), &env, &info, &token, amount)?;
            token_idx = cooperative
                .whitelisted_tokens
                .iter()
                .position(|x| x.contract_addr == Some(Addr::unchecked(&token)));

            messages = [messages, validate_msg].concat();
        }

        if token_idx.is_none() {
            return Err(ContractError::InvalidToken {});
        }
        let token_idx = token_idx.unwrap();

        let whitelisted_token = cooperative.whitelisted_tokens.get(token_idx);

        if whitelisted_token.is_none() {
            return Err(ContractError::InvalidToken {});
        }

        let whitelisted_token = whitelisted_token.unwrap();

        // Handle token transfer based on type
        if whitelisted_token.is_native {
            // For native tokens, verify sent funds
            // Create bank transfer message
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: env.contract.address.to_string(),
                amount: vec![Coin {
                    denom: token.clone(),
                    amount,
                }],
            }));
        } else {
            // For CW20 tokens
            let cw20_addr = whitelisted_token
                .contract_addr
                .clone()
                .ok_or(ContractError::InvalidToken {})?;

            // Create CW20 transfer message
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&cw20::Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount,
                })?,
                funds: vec![],
            }));
        }

        // Find member
        let member_idx = cooperative
            .members
            .iter()
            .position(|m| m.address == info.sender)
            .ok_or(ContractError::MemberNotFound {})?;

        //Update member contribution
        let mut current_amount: Uint128 = Uint128::zero();
        cooperative.members[member_idx]
            .contribution
            .iter()
            .find(|x| x.0 == token_idx as u64)
            .map(|x| {
                current_amount = x.1.into();
            });

        current_amount = current_amount + amount;

        cooperative.members[member_idx]
            .contribution
            .push((token_idx as u64, current_amount));
        let member = cooperative.members.get(member_idx);
        if member.is_none() {
            return Err(ContractError::MemberNotFound {});
        }

        // Update cooperative total funds
        let current_token_idx = STATE.load(deps.storage)?.current_whitelisted_token_id;
        let fund = cooperative.total_funds.get(token_idx);

        if fund.is_none() {
            cooperative.total_funds.push((current_token_idx, amount));
        } else {
            cooperative.total_funds[token_idx].1 += amount;
        };

        COOPERATIVES.save(deps.storage, cooperative_name.clone(), &cooperative)?;

        // Update state total pooled funds
        let mut state = STATE.load(deps.storage)?;
        let state_token_idx = state
            .total_pooled_funds
            .iter()
            .position(|(id, _)| *id == token_idx as u64)
            .unwrap_or_else(|| state.total_pooled_funds.len());

        if state_token_idx == state.total_pooled_funds.len() {
            state.total_pooled_funds.push((token_idx as u64, amount));
        } else {
            state.total_pooled_funds[state_token_idx].1 += amount;
        }

        STATE.save(deps.storage, &state)?;

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "fund_cooperative")
            .add_attribute("cooperative", cooperative_name)
            .add_attribute("token", token)
            .add_attribute("amount", amount.to_string()))
    }

    fn validate_cw20(
        deps: &Deps,
        env: &Env,
        info: &MessageInfo,
        token_address: &str,
        required_amount: Uint128,
    ) -> StdResult<Vec<CosmosMsg>> {
        // Query token balance
        let balance: cw20::BalanceResponse = deps.querier.query_wasm_smart(
            token_address,
            &cw20::Cw20QueryMsg::Balance {
                address: info.sender.to_string(),
            },
        )?;

        // Check if user has sufficient balance
        if balance.balance < required_amount {
            return Err(StdError::generic_err(format!(
                "Insufficient CW20 token balance. Required: {}, Balance: {}",
                required_amount, balance.balance
            )));
        }

        // Query allowance
        let allowance: cw20::AllowanceResponse = deps.querier.query_wasm_smart(
            token_address,
            &cw20::Cw20QueryMsg::Allowance {
                owner: info.sender.to_string(),
                spender: env.contract.address.to_string(),
            },
        )?;

        // Check if contract has sufficient allowance
        if allowance.allowance < required_amount {
            return Err(StdError::generic_err(format!(
                "Insufficient CW20 token allowance. Required: {}, Allowance: {}",
                required_amount, allowance.allowance
            )));
        }

        // move funds to contract
        let mut messages: Vec<CosmosMsg> = vec![];
        messages.push(
            WasmMsg::Execute {
                contract_addr: token_address.to_string(),
                msg: to_json_binary(&cw20::Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: required_amount,
                })?,
                funds: vec![],
            }
            .into(),
        );

        Ok(messages)
    }

    pub fn execute_borrow(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cooperative_name: String,
        tokens_in: Vec<Addr>,
        amount_in: Vec<Uint128>,
        token_out: Addr,
        min_amount_out: Uint128,
    ) -> Result<Response, ContractError> {
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let mut cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;
        let state = STATE.load(deps.storage)?;
        let loan_id = state.current_loan_id;

        // Find member
        let member_idx = cooperative
            .members
            .iter()
            .position(|m| m.address == info.sender)
            .ok_or(ContractError::MemberNotFound {})?;

        // Calculate amount out
        let member = &mut cooperative.clone().members[member_idx];
        let mut collateral_value: Uint128 = 0u128.into();
        let mut amount_out: Uint128 = 0u128.into();

        let mut messages: Vec<CosmosMsg> = vec![];

        let mut member_iter = member.clone();
        let contribution = member_iter.contribution.iter_mut();
        for (idx, (contribution_token_id, _)) in contribution.enumerate() {
            for token in tokens_in.iter() {
                let token_id = TOKENS.load(deps.storage, token.clone())?;

                if contribution_token_id == &token_id {
                    let c_value = PRICES.load(deps.storage, token_id)?;

                    collateral_value += c_value.latest_price_to_usd;

                    let asset_price = PRICES.load(deps.storage, token_id)?;
                    let a_value = amount_in[idx]
                        * collateral_value
                        * cooperative
                            .risk_profile
                            .collateralization_ratio
                            .to_uint_floor();

                    amount_out = a_value / asset_price.latest_price_to_usd;

                    // update contribution to prevent double spending
                    let token_idx = member
                        .clone()
                        .contribution
                        .iter()
                        .position(|x| x.0 == token_id);
                    if token_idx.is_none() {
                        return Err(ContractError::InvalidToken {});
                    }

                    let mut part = member.contribution[token_idx.unwrap()];
                    part.1 -= part.1.checked_sub(amount_in[idx]).unwrap();
                    cooperative.members[member_idx].contribution[token_idx.unwrap()] = part;
                }
            }
        }

        if amount_out < min_amount_out {
            return Err(ContractError::InsufficientCollateral {});
        }

        // Create new loan
        let rate: u128 = cooperative.risk_profile.interest_rate.to_uint_ceil().into();
        let loan = Loan {
            id: loan_id,
            amount: amount_out,
            token: token_out.clone(),
            collaterals: tokens_in,
            collaterals_amount: amount_in,
            interest_rate: Decimal::percent(rate as u64),
            status: LoanStatus::Active,
        };

        let w_token_idx = cooperative
            .whitelisted_tokens
            .iter()
            .position(|x| x.contract_addr == Some(token_out.clone()));

        if w_token_idx.is_none() {
            return Err(ContractError::InvalidToken {});
        }
        let w_token = cooperative
            .whitelisted_tokens
            .get(w_token_idx.unwrap())
            .unwrap();
        let cw20_addr = w_token.clone().contract_addr;

        // For native tokens
        if cw20_addr.is_none() {
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin {
                    denom: w_token.denom.clone(),
                    amount: amount_out,
                }],
            }));
        }

        // Create CW20 transfer message
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cw20_addr.unwrap().to_string(),
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.to_string(),
                recipient: env.contract.address.to_string(),
                amount: amount_out,
            })?,
            funds: vec![],
        }));

        cooperative.members[member_idx].active_loans.push(loan);
        COOPERATIVES.save(deps.storage, cooperative_name, &cooperative)?;

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "borrow")
            .add_attribute("token", "denom")
            .add_attribute("amount_out", amount_out.to_string()))
    }

    pub fn execute_repay(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cooperative_name: String,
        token: Addr,
    ) -> Result<Response, ContractError> {
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let mut cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;

        // Find member
        let member_idx = cooperative
            .members
            .iter()
            .position(|m| m.address == info.sender)
            .ok_or(ContractError::MemberNotFound {})?;

        // Find active loan
        let loan_idx = cooperative.members[member_idx]
            .active_loans
            .iter()
            .position(|l| l.status == LoanStatus::Active)
            .ok_or(ContractError::NoActiveLoan {})?;

        let loan = &mut cooperative.members[member_idx].active_loans[loan_idx];
        loan.status = LoanStatus::Repaid;

        let amount = loan.amount;

        // find token
        let w_token_idx = cooperative
            .whitelisted_tokens
            .iter()
            .position(|x| x.contract_addr == Some(token.clone()));

        if w_token_idx.is_none() {
            return Err(ContractError::InvalidToken {});
        }
        let w_token = cooperative
            .whitelisted_tokens
            .get(w_token_idx.unwrap())
            .unwrap();

        // return collaterals
        let collaterals = cooperative.members[member_idx].active_loans[loan_idx]
            .collaterals
            .clone();

        for (idx, collateral) in collaterals.iter().enumerate() {
            let token_id = TOKENS.load(deps.storage, collateral.clone())?;
            let token_idx = cooperative
                .whitelisted_tokens
                .iter()
                .position(|x| x.contract_addr == Some(collateral.clone()));
            if token_idx.is_none() {
                return Err(ContractError::InvalidToken {});
            }
            let bution_idx = cooperative.members[member_idx]
                .contribution
                .iter()
                .position(|x| x.0 == token_id)
                .unwrap();

            let amount = cooperative.members[member_idx].contribution[bution_idx].1
                + cooperative.members[member_idx].active_loans[loan_idx].collaterals_amount[idx];
            cooperative.members[member_idx].contribution[bution_idx] =
                (token_idx.unwrap() as u64, amount);
        }

        // transfer token back to contract
        let mut messages: Vec<CosmosMsg> = vec![];

        let sent_funds = info
            .funds
            .iter()
            .find(|coin| coin.denom == w_token.denom)
            .ok_or(ContractError::NoFunds {})?;

        if sent_funds.amount != amount {
            return Err(ContractError::InvalidFundAmount {});
        }

        // Handle token transfer based on type
        if w_token.is_native {
            // For native tokens, verify sent funds
            // Create bank transfer message
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: env.contract.address.to_string(),
                amount: vec![Coin {
                    denom: w_token.denom.clone(),
                    amount,
                }],
            }));
        } else {
            // For CW20 tokens
            let cw20_addr = w_token
                .clone()
                .contract_addr
                .ok_or(ContractError::InvalidToken {})?;

            // Create CW20 transfer message
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&cw20::Cw20ExecuteMsg::TransferFrom {
                    owner: env.contract.address.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount,
                })?,
                funds: vec![],
            }));
        }

        COOPERATIVES.save(deps.storage, cooperative_name, &cooperative)?;

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "repay")
            .add_attribute("amount", amount.to_string()))
    }

    pub fn withdraw_contribution_and_rewards(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cooperative_name: String,
        token: Addr,
    ) -> Result<Response, ContractError> {
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let mut cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;
        let member_idx = cooperative
            .members
            .iter()
            .position(|m| m.address == info.sender)
            .ok_or(ContractError::MemberNotFound {})?;

        // find token
        let w_token_idx = cooperative
            .whitelisted_tokens
            .iter()
            .position(|x| x.contract_addr == Some(token.clone()));

        if w_token_idx.is_none() {
            return Err(ContractError::InvalidToken {});
        }
        let w_token = cooperative
            .whitelisted_tokens
            .get(w_token_idx.unwrap())
            .unwrap();

        let contribution_idx = cooperative.members[member_idx]
            .contribution
            .iter()
            .position(|x| x.0 == w_token_idx.unwrap() as u64);
        let contribution =
            &mut cooperative.members[member_idx].contribution[contribution_idx.unwrap()].clone();
        let amount = contribution.1;

        // Check if the total funds available in the cooperative are sufficient
        let total_available_funds = cooperative.total_funds[w_token_idx.unwrap()].1;
        if total_available_funds < amount {
            return Err(ContractError::InsufficientPoolFunds {});
        }

        // Calculate the member's share of rewards based on the current contribution
        let total_pooled_funds = cooperative.total_funds[w_token_idx.unwrap()].1;
        let member_share = if total_pooled_funds.is_zero() {
            Uint128::zero()
        } else {
            let rewards_pool_key = (cooperative_name.clone(), w_token_idx.unwrap() as u64);
            let rewards_pool = REWARDS_POOLS.load(deps.storage, rewards_pool_key)?;
            (contribution.1 * rewards_pool.total_rewards) / total_pooled_funds
        };

        // Check if the member has sufficient rewards
        let share = &mut cooperative.members[member_idx].share[w_token_idx.unwrap()];
        if share.1 < member_share {
            return Err(ContractError::InsufficientRewards {});
        }

        // Update contribution and share
        contribution.1 = 0u128.into();
        share.1 = 0u128.into();

        // Update cooperative total funds
        cooperative.total_funds[w_token_idx.unwrap()].1 -= amount;

        // Update the rewards pool's distributed rewards
        let mut rewards_pool = REWARDS_POOLS.load(
            deps.storage,
            (cooperative_name.clone(), w_token_idx.unwrap() as u64),
        )?;
        rewards_pool.distributed_rewards += member_share;
        REWARDS_POOLS.save(
            deps.storage,
            (cooperative_name.clone(), w_token_idx.unwrap() as u64),
            &rewards_pool,
        )?;

        // Save updated cooperative data
        COOPERATIVES.save(deps.storage, cooperative_name.clone(), &cooperative)?;

        // Handle token transfer based on type
        let mut messages: Vec<CosmosMsg> = vec![];
        if cooperative.whitelisted_tokens[w_token_idx.unwrap()].is_native {
            // For native tokens, create bank transfer message
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin {
                    denom: w_token.denom.clone(),
                    amount,
                }],
            }));
        } else {
            // For CW20 tokens
            let cw20_addr = cooperative.whitelisted_tokens[w_token_idx.unwrap()]
                .contract_addr
                .clone()
                .ok_or(ContractError::InvalidToken {})?;

            // Create CW20 transfer message
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&cw20::Cw20ExecuteMsg::TransferFrom {
                    owner: env.contract.address.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount,
                })?,
                funds: vec![],
            }));
        }

        COOPERATIVES.save(deps.storage, cooperative_name, &cooperative)?;

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "repay")
            .add_attribute("amount", amount.to_string()))
    }

    pub fn execute_propose(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        cooperative_name: String,
        proposal: Proposal,
    ) -> Result<Response, ContractError> {
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;

        // Check if member exists
        if !cooperative.members.iter().any(|m| m.address == info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        let mut state = STATE.load(deps.storage)?;
        let proposal_id = state.current_proposal_id + 1;

        // Ensure required data was supplied
        if proposal.proposal_type == ProposalType::AddMember {
            assert!(proposal.data.new_member_addr.is_some());
        } else if proposal.proposal_type == ProposalType::WhitelistToken {
            assert!(proposal.data.denom.is_some());
            match proposal.data.is_native {
                Some(is_native) => {
                    if !is_native {
                        assert!(proposal.data.token_addr.is_some());
                    }
                }
                None => {
                    assert!(proposal.data.token_addr.is_none());
                }
            }
        }

        // Construct proposal with default values
        let proposal = Proposal {
            id: proposal_id,
            description: proposal.description,
            data: proposal.data,
            votes: vec![],
            aye_count: 0,
            nay_count: 0,
            aye_weights: 0,
            nay_weights: 0,
            end_time: proposal.end_time,
            quorum: proposal.quorum,
            proposal_type: proposal.proposal_type,
            outcome: None,
            executed: false,
        };

        // Update storage
        state.current_proposal_id = proposal_id;
        STATE.save(deps.storage, &state)?;

        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

        let mut coop_proposals = COOPERATIVES_PROPOSALS
            .load(deps.storage, cooperative_name.clone())
            .unwrap_or_default();
        coop_proposals.push(proposal_id);
        COOPERATIVES_PROPOSALS.save(deps.storage, cooperative_name, &coop_proposals)?;

        Ok(Response::new()
            .add_attribute("action", "propose")
            .add_attribute("proposal_id", proposal_id.to_string()))
    }

    pub fn execute_vote(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cooperative_name: String,
        proposal_id: u64,
        weight: Uint128,
        aye: bool,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;

        // Check if proposal has ended
        if proposal.outcome.is_some() {
            return Err(ContractError::ProposalEnded {});
        }

        // Check if member has already voted
        if proposal.votes.iter().any(|v| v.voter == info.sender) {
            return Err(ContractError::AlreadyVoted {});
        }

        // Record vote
        let vote = Vote {
            voter: info.sender,
            conviction: weight,
            voted_at: env.block.time.seconds(),
        };
        proposal.votes.push(vote);

        if aye {
            proposal.aye_count += 1;
            proposal.aye_weights += weight.u128() as u64;
        } else {
            proposal.nay_count += 1;
            proposal.nay_weights += weight.u128() as u64;
        }

        let cw20_contract_addr = state.weight_token;
        let transfer_msg = Cw20ExecuteMsg::Transfer {
            recipient: env.contract.address.into_string(),
            amount: weight,
        };

        let msg: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cw20_contract_addr.to_string(),
            msg: to_json_binary(&transfer_msg)?,
            funds: vec![],
        })];

        // Check if proposal can be finalized
        if let Some(quorum) = proposal.quorum {
            let total_votes = proposal.aye_weights + proposal.nay_weights;
            if Decimal::from_ratio(total_votes, 100u64) >= quorum {
                proposal.outcome = Some(if proposal.aye_weights > proposal.nay_weights {
                    ProposalOutcome::Passed
                } else {
                    ProposalOutcome::Rejected
                });
            }
        }

        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

        Ok(Response::new()
            .add_messages(msg)
            .add_attribute("action", "vote")
            .add_attribute("cooperative_name", cooperative_name)
            .add_attribute("proposal_id", proposal_id.to_string()))
    }

    pub fn execute_withdraw_weight(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cooperative_name: String,
        proposal_id: u64,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;

        // Check proposal has not ended
        if proposal.outcome.is_none() {
            return Err(ContractError::ProposalInProcess {});
        }

        // Get user weight
        let weight = proposal.votes.iter().find(|x| x.voter == info.sender);

        if weight.is_none() {
            return Err(ContractError::NoWeightsToWithdraw {});
        }

        let weight = weight.unwrap().conviction;

        let cw20_contract_addr = state.weight_token;
        let transfer_msg = Cw20ExecuteMsg::Transfer {
            recipient: env.contract.address.into_string(),
            amount: weight,
        };

        let msg: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cw20_contract_addr.to_string(),
            msg: to_json_binary(&transfer_msg)?,
            funds: vec![],
        })];

        // Update weight to prevent double spending
        let idx = proposal.votes.iter().position(|x| x.voter == info.sender);
        proposal.votes[idx.unwrap()].conviction = 0u128.into();

        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

        Ok(Response::new()
            .add_messages(msg)
            .add_attribute("action", "withdraw weight")
            .add_attribute("cooperative_name", cooperative_name)
            .add_attribute("proposal_id", proposal_id.to_string()))
    }

    pub fn execute_proposal(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cooperative_name: String,
        proposal_id: u64,
    ) -> Result<Response, ContractError> {
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;
        let proposal = PROPOSALS.load(deps.storage, proposal_id)?;

        // Ensure proposal has not been executed
        if proposal.executed {
            return Err(ContractError::ProposalAlreadyExecuted {});
        }

        // Check if signer is member
        if !cooperative
            .members
            .iter()
            .any(|m| m.address == info.sender.clone())
        {
            return Err(ContractError::Unauthorized {});
        }

        // Check proposal has ended
        if env.block.time.seconds() >= proposal.end_time {
            return Err(ContractError::ProposalInProcess {});
        }

        // Check proposal has passed
        match proposal.outcome {
            Some(outcome) => {
                if outcome == ProposalOutcome::Rejected {
                    return Err(ContractError::ProposalRejected {});
                } else {
                    let proposal_type = proposal.proposal_type;
                    let proposal_data = proposal.data;
                    match proposal_type {
                        ProposalType::WhitelistToken => {
                            let res = execute_add_whitelisted_token(
                                deps,
                                info.clone(),
                                proposal_id,
                                cooperative_name,
                                proposal_data.denom.unwrap(),
                                Some(proposal_data.token_addr.unwrap()),
                                proposal_data.is_native.unwrap(),
                                proposal_data.max_loan_ratio.unwrap(),
                            )?;
                            return Ok(res);
                        }
                        ProposalType::AddMember => {
                            let res = execute_add_member(
                                deps,
                                env,
                                info.clone(),
                                proposal_id,
                                cooperative_name,
                                proposal_data.new_member_addr.unwrap(),
                            )?;
                            return Ok(res);
                        }
                        _ => {
                            return Err(ContractError::NotImplemented {});
                        }
                    }
                }
            }
            None => {
                return Err(ContractError::ProposalInProcess {});
            }
        }
    }

    fn execute_add_whitelisted_token(
        deps: DepsMut,
        info: MessageInfo,
        proposal_id: u64,
        cooperative_name: String,
        denom: String,
        contract_addr: Option<Addr>,
        is_native: bool,
        max_loan_ratio: Decimal,
    ) -> Result<Response, ContractError> {
        let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;
        let mut cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;
        let mut state = STATE.load(deps.storage)?;
        let whitelisted_token_id = state.current_whitelisted_token_id + 1;

        // Verify caller is contract owner
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }

        // Check not already whitelisted
        if cooperative
            .whitelisted_tokens
            .iter()
            .any(|m| m.denom == denom)
        {
            return Err(ContractError::TokenAlreadyWhitelisted {});
        }

        // Ensure whitelisted tokens is less than or equal to 5.
        assert!(
            cooperative.whitelisted_tokens.len() <= 5,
            "{}",
            ContractError::MaxWhitelistedTokensReached {}
        );

        // Create new whitelisted token
        let token = WhitelistedToken {
            denom,
            contract_addr,
            is_native,
            max_loan_ratio,
        };

        // Update storage
        state.current_whitelisted_token_id = whitelisted_token_id;
        STATE.save(deps.storage, &state)?;

        cooperative.whitelisted_tokens.push(token.clone());
        COOPERATIVES.save(
            deps.storage,
            cooperative_name.trim().to_lowercase(),
            &cooperative,
        )?;

        proposal.executed = true;
        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

        Ok(Response::new()
            .add_attribute("action", "add_whitelisted_token")
            .add_attribute("token", token.denom))
    }

    fn execute_add_member(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        proposal_id: u64,
        cooperative_name: String,
        new_member_addr: Addr,
    ) -> Result<Response, ContractError> {
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;
        let mut cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;

        // Check if member already exists
        if cooperative
            .members
            .iter()
            .any(|m| m.address == new_member_addr)
        {
            return Err(ContractError::AlreadyMember {});
        }

        let new_member = Member {
            address: new_member_addr.clone(),
            contribution: vec![],
            share: vec![],
            joined_at: env.block.time.seconds(),
            reputation_score: Decimal::zero(),
            active_loans: vec![],
        };

        cooperative.members.push(new_member);
        COOPERATIVES.save(deps.storage, cooperative_name.clone(), &cooperative)?;

        // Update member's cooperative list
        let mut member_coops = MEMBERS
            .load(deps.storage, info.sender.clone())
            .unwrap_or_default();
        member_coops.push(cooperative_name.clone());
        MEMBERS.save(deps.storage, new_member_addr, &member_coops)?;

        // Update proposal
        proposal.executed = true;
        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

        Ok(Response::new()
            .add_attribute("action", "join_cooperative")
            .add_attribute("cooperative", cooperative_name))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCooperative { cooperative_name } => {
            to_json_binary(&query::get_cooperative(deps, cooperative_name)?)
        }
        QueryMsg::GetMemberInfo {
            cooperative_name,
            member,
        } => to_json_binary(&query::get_member_info(deps, cooperative_name, member)?),
        QueryMsg::GetProposal { proposal_id } => {
            to_json_binary(&query::get_proposal(deps, proposal_id)?)
        }
        QueryMsg::GetWhitelistedTokens { cooperative_name } => {
            to_json_binary(&query::get_whitelisted_tokens(deps, cooperative_name)?)
        }
        QueryMsg::ListCooperatives { min, max } => {
            to_json_binary(&query::list_cooperative(deps, min, max)?)
        }
        QueryMsg::GetTokenId { token } => to_json_binary(&query::get_token_id(deps, token)?),
    }
}

pub mod query {

    use cosmwasm_std::StdError;
    use cw_storage_plus::Bound;

    use crate::msg::{
        GetCooperativeResponse, GetListCooperativesResponse, GetMemberInfoResponse,
        GetProposalResponse, GetTokenIdResponse, GetWhitelistedTokensResponse,
    };

    use super::*;

    pub fn get_cooperative(
        deps: Deps,
        cooperative_name: String,
    ) -> StdResult<GetCooperativeResponse> {
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let corporative = COOPERATIVES.load(deps.storage, cooperative_name)?;

        Ok(GetCooperativeResponse { corporative })
    }

    pub fn get_member_info(
        deps: Deps,
        cooperative_name: String,
        member: Addr,
    ) -> StdResult<GetMemberInfoResponse> {
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let corporative = COOPERATIVES.load(deps.storage, cooperative_name)?;

        let member_idx = corporative.members.iter().position(|x| x.address == member);

        if member_idx.is_none() {
            return Ok(GetMemberInfoResponse {
                info: Member {
                    address: Addr::unchecked("0"),
                    contribution: vec![],
                    share: vec![],
                    joined_at: 0,
                    reputation_score: Decimal::zero(),
                    active_loans: vec![],
                },
            });
        }

        let member_idx = member_idx.unwrap();
        let member_data = corporative.members.get(member_idx);

        if member_data.is_none() {
            return Ok(GetMemberInfoResponse {
                info: Member {
                    address: Addr::unchecked("0"),
                    contribution: vec![],
                    share: vec![],
                    joined_at: 0,
                    reputation_score: Decimal::zero(),
                    active_loans: vec![],
                },
            });
        }

        let info = member_data.unwrap();
        Ok(GetMemberInfoResponse { info: info.clone() })
    }

    pub fn get_proposal(deps: Deps, proposal_id: u64) -> StdResult<GetProposalResponse> {
        let proposal = PROPOSALS.load(deps.storage, proposal_id)?;

        Ok(GetProposalResponse { proposal })
    }

    pub fn get_whitelisted_tokens(
        deps: Deps,
        cooperative_name: String,
    ) -> StdResult<GetWhitelistedTokensResponse> {
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let corporative = COOPERATIVES.load(deps.storage, cooperative_name)?;
        let tokens = corporative.whitelisted_tokens;

        Ok(GetWhitelistedTokensResponse { tokens })
    }

    pub fn list_cooperative(
        deps: Deps,
        min: String,
        max: String,
    ) -> StdResult<GetListCooperativesResponse> {
        let cooperatives = COOPERATIVES
            .keys(
                deps.storage,
                Some(Bound::inclusive(min)),
                Some(Bound::inclusive(max)),
                Order::Descending,
            )
            .collect::<Result<Vec<String>, StdError>>()?;

        Ok(GetListCooperativesResponse { cooperatives })
    }

    pub fn get_token_id(deps: Deps, token: String) -> StdResult<GetTokenIdResponse> {
        let token_id = TOKENS.load(deps.storage, Addr::unchecked(token))?;

        Ok(GetTokenIdResponse { token_id })
    }
}

#[cfg(test)]
mod tests {}
