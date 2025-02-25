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
        weight_token: "untrn".to_string(),
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
        usd_price: Decimal,
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
        let current_token_idx = state.current_whitelisted_token_id + 1;

        // add whitelisted token to
        for token in initial_whitelisted_tokens {
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
        state.current_whitelisted_token_id = current_token_idx;

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
        // Normalize cooperative name
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let mut cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;
        let mut messages: Vec<CosmosMsg> = vec![];

        // Validate transfer and find token index
        let token_idx = if is_native {
            // Validate native token
            let coin = info
                .funds
                .iter()
                .find(|coin| coin.denom == token)
                .ok_or(ContractError::NoFunds {})?;

            if coin.amount != amount {
                return Err(ContractError::FundsMustMatchAmount {});
            }

            // Find token in whitelist
            cooperative
                .whitelisted_tokens
                .iter()
                .position(|x| x.denom == coin.denom && x.is_native)
                .ok_or(ContractError::InvalidToken {})?
        } else {
            // Validate CW20 token
            let validate_msg = validate_cw20(&deps.as_ref(), &env, &info, &token, amount)?;
            messages = [messages, validate_msg].concat();

            // Find token in whitelist
            cooperative
                .whitelisted_tokens
                .iter()
                .position(|x| !x.is_native && x.contract_addr == Some(Addr::unchecked(&token)))
                .ok_or(ContractError::InvalidToken {})?
        };

        // Get the whitelisted token (we know it exists now)
        let whitelisted_token = &cooperative.whitelisted_tokens[token_idx];

        // Handle token cw20 transfer
        if !whitelisted_token.is_native {
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

        // Update member contribution more efficiently
        let token_id = token_idx as u64;
        let contribution_idx = cooperative.members[member_idx]
            .contribution
            .iter()
            .position(|(id, _)| *id == token_id);

        if let Some(idx) = contribution_idx {
            // Update existing contribution
            cooperative.members[member_idx].contribution[idx].1 += amount;
        } else {
            // Add new contribution
            cooperative.members[member_idx]
                .contribution
                .push((token_id, amount));
        }

        // Update cooperative total funds more efficiently
        let fund_idx = cooperative
            .total_funds
            .iter()
            .position(|(id, _)| *id == token_id);

        if let Some(idx) = fund_idx {
            // Update existing fund
            cooperative.total_funds[idx].1 += amount;
        } else {
            // Add new fund
            cooperative.total_funds.push((token_id, amount));
        }

        COOPERATIVES.save(deps.storage, cooperative_name.clone(), &cooperative)?;

        // Update state total pooled funds
        let mut state = STATE.load(deps.storage)?;
        let state_token_idx = state
            .total_pooled_funds
            .iter()
            .position(|(id, _)| *id == token_id);

        if let Some(idx) = state_token_idx {
            // Update existing pooled funds
            state.total_pooled_funds[idx].1 += amount;
        } else {
            // Add new pooled funds
            state.total_pooled_funds.push((token_id, amount));
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
        // Validate input arrays
        if tokens_in.is_empty() || amount_in.is_empty() || tokens_in.len() != amount_in.len() {
            return Err(ContractError::InvalidInput {});
        }

        // Normalize cooperative name
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let mut cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;
        let mut state = STATE.load(deps.storage)?;
        let loan_id = state.current_loan_id;

        // Find member
        let member_idx = cooperative
            .members
            .iter()
            .position(|m| m.address == info.sender)
            .ok_or(ContractError::MemberNotFound {})?;

        // Validate output token
        let w_token_idx = cooperative
            .whitelisted_tokens
            .iter()
            .position(|x| x.contract_addr == Some(token_out.clone()))
            .ok_or(ContractError::InvalidToken {})?;

        let w_token = &cooperative.whitelisted_tokens[w_token_idx];

        // Calculate collateral value and amount out
        let mut collateral_value: Uint128 = Uint128::zero();
        let mut collateral_details: Vec<(u64, Uint128)> = Vec::new();
        let mut messages: Vec<CosmosMsg> = vec![];

        // Process each input token
        for (idx, token) in tokens_in.iter().enumerate() {
            let token_id = TOKENS.load(deps.storage, token.clone())?;

            // Find token in member's contributions
            let contribution_idx = cooperative.members[member_idx]
                .contribution
                .iter()
                .position(|x| x.0 == token_id)
                .ok_or(ContractError::NoContribution {})?;

            let available_amount = cooperative.members[member_idx].contribution[contribution_idx].1;
            let requested_amount = amount_in[idx];

            // Ensure member has enough of this token
            if available_amount < requested_amount {
                return Err(ContractError::InsufficientFunds {});
            }

            // Get token price for valuation
            let token_price = PRICES.load(deps.storage, token_id)?;
            let token_value = requested_amount * token_price.latest_price_to_usd.to_uint_floor();

            collateral_value += token_value;
            collateral_details.push((token_id, requested_amount));

            // Reduce member's contribution
            cooperative.members[member_idx].contribution[contribution_idx].1 -= requested_amount;
        }

        // Calculate amount out based on collateral value and risk profile
        let token_out_id = TOKENS.load(deps.storage, token_out.clone())?;
        let token_out_price = PRICES.load(deps.storage, token_out_id)?;

        let loan_value = collateral_value
            * cooperative
                .risk_profile
                .collateralization_ratio
                .to_uint_floor();
        let amount_out = loan_value / token_out_price.latest_price_to_usd.to_uint_floor();

        // Ensure minimum amount out is met
        if amount_out < min_amount_out {
            return Err(ContractError::InsufficientCollateral {});
        }

        // Create new loan
        let interest_rate = cooperative.risk_profile.interest_rate.to_uint_ceil().u128() as u64;
        let loan = Loan {
            id: loan_id,
            amount: amount_out,
            token: token_out.clone(),
            collaterals: tokens_in.clone(),
            collaterals_amount: amount_in.clone(),
            interest_rate: Decimal::percent(interest_rate),
            status: LoanStatus::Active,
        };

        // Handle token transfer based on type
        if w_token.is_native {
            // Check if the contract has enough funds
            let contract_balance = deps
                .querier
                .query_balance(env.contract.address.clone(), w_token.denom.clone())?;

            if contract_balance.amount < amount_out {
                return Err(ContractError::InsufficientPoolFunds {});
            }

            // For native tokens
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin {
                    denom: w_token.denom.clone(),
                    amount: amount_out,
                }],
            }));
        } else {
            // For CW20 tokens
            let cw20_addr = w_token
                .contract_addr
                .clone()
                .ok_or(ContractError::InvalidToken {})?;

            // Check contract balance
            let balance: cw20::BalanceResponse = deps.querier.query_wasm_smart(
                cw20_addr.clone(),
                &cw20::Cw20QueryMsg::Balance {
                    address: env.contract.address.to_string(),
                },
            )?;

            if balance.balance < amount_out {
                return Err(ContractError::InsufficientPoolFunds {});
            }

            // Create CW20 transfer message
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&cw20::Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount: amount_out,
                })?,
                funds: vec![],
            }));
        }

        // Update member data
        cooperative.members[member_idx].loans.push(loan);

        // Update state
        state.current_loan_id += 1;

        // Save updates
        COOPERATIVES.save(deps.storage, cooperative_name, &cooperative)?;
        STATE.save(deps.storage, &state)?;

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "borrow")
            .add_attribute("loan_id", loan_id.to_string())
            .add_attribute("borrower", info.sender.to_string())
            .add_attribute("token_out", token_out.to_string())
            .add_attribute("amount_out", amount_out.to_string())
            .add_attribute("collateral_value", collateral_value.to_string()))
    }

    pub fn execute_repay(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cooperative_name: String,
        token: Addr,
    ) -> Result<Response, ContractError> {
        // Normalize cooperative name
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let mut cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;

        // Find member
        let member_idx = cooperative
            .members
            .iter()
            .position(|m| m.address == info.sender)
            .ok_or(ContractError::MemberNotFound {})?;

        // Find active loan (for given token)
        let loan_idx = cooperative.members[member_idx]
            .loans
            .iter()
            .position(|l| l.status == LoanStatus::Active && l.token == token)
            .ok_or(ContractError::NoActiveLoan {})?;

        let loan = cooperative.members[member_idx].loans[loan_idx].clone();
        let repayment_amount = loan.amount;

        // Find token in whitelist
        let w_token_idx = cooperative
            .whitelisted_tokens
            .iter()
            .position(|x| x.contract_addr == Some(token.clone()))
            .ok_or(ContractError::InvalidToken {})?;

        let w_token = &cooperative.whitelisted_tokens[w_token_idx];

        // Validate payment
        if w_token.is_native {
            // For native tokens, verify sent funds
            let sent_funds = info
                .funds
                .iter()
                .find(|coin| coin.denom == w_token.denom)
                .ok_or(ContractError::NoFunds {})?;

            if sent_funds.amount != repayment_amount {
                return Err(ContractError::InvalidFundAmount {});
            }
        } else {
            // For CW20 tokens, validate allowance first
            let cw20_addr = w_token
                .contract_addr
                .clone()
                .ok_or(ContractError::InvalidToken {})?;

            // Check allowance
            let allowance: cw20::AllowanceResponse = deps.querier.query_wasm_smart(
                cw20_addr.clone(),
                &cw20::Cw20QueryMsg::Allowance {
                    owner: info.sender.to_string(),
                    spender: env.contract.address.to_string(),
                },
            )?;

            if allowance.allowance < repayment_amount {
                return Err(ContractError::InsufficientAllowance {});
            }
        }

        // Return collaterals to the member's contribution
        let collaterals = &loan.collaterals;
        let collateral_amounts = &loan.collaterals_amount;

        // Here's the fixed version of your loan repayment code with proper bounds checking
        for (idx, collateral) in collaterals.iter().enumerate() {
            // Add bounds check to prevent index out of bounds error
            if idx >= collateral_amounts.len() {
                return Err(ContractError::InvalidCollateral {
                    msg: format!("Collateral amount missing for collateral at index {}", idx),
                });
            }

            let token_id = TOKENS.load(deps.storage, collateral.clone())?;

            // Find or create contribution entry
            let contribution_idx = cooperative.members[member_idx]
                .contribution
                .iter()
                .position(|x| x.0 == token_id);

            if let Some(contrib_idx) = contribution_idx {
                // Update existing contribution
                cooperative.members[member_idx].contribution[contrib_idx].1 +=
                    collateral_amounts[idx];
            } else {
                // Add new contribution
                cooperative.members[member_idx]
                    .contribution
                    .push((token_id, collateral_amounts[idx]));
            }
        }

        // Mark loan as repaid
        cooperative.members[member_idx].loans[loan_idx].status = LoanStatus::Repaid;

        // Handle token transfer based on type
        let mut messages: Vec<CosmosMsg> = vec![];

        if !w_token.is_native {
            // For CW20 tokens
            let cw20_addr = w_token
                .contract_addr
                .clone()
                .ok_or(ContractError::InvalidToken {})?;

            // Create CW20 transferFrom message
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: repayment_amount,
                })?,
                funds: vec![],
            }));
        }

        // Save updated cooperative data
        COOPERATIVES.save(deps.storage, cooperative_name, &cooperative)?;

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "repay")
            .add_attribute("loan_id", loan.id.to_string())
            .add_attribute("borrower", info.sender.to_string())
            .add_attribute("token", token.to_string())
            .add_attribute("amount", repayment_amount.to_string()))
    }

    pub fn withdraw_contribution_and_rewards(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        cooperative_name: String,
        token: Addr,
    ) -> Result<Response, ContractError> {
        // Normalize cooperative name
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let mut cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;

        // Find member
        let member_idx = cooperative
            .members
            .iter()
            .position(|m| m.address == info.sender)
            .ok_or(ContractError::MemberNotFound {})?;

        // Find token in whitelist
        let w_token_idx = cooperative
            .whitelisted_tokens
            .iter()
            .position(|x| x.contract_addr == Some(token.clone()))
            .ok_or(ContractError::InvalidToken {})?;

        let w_token = &cooperative.whitelisted_tokens[w_token_idx];

        // Find member's contribution for this token
        let contribution_idx = cooperative.members[member_idx]
            .contribution
            .iter()
            .position(|x| x.0 == w_token_idx as u64)
            .ok_or(ContractError::NoContribution {})?;

        // Get the amount to withdraw
        let amount = cooperative.members[member_idx].contribution[contribution_idx].1;
        if amount.is_zero() {
            return Err(ContractError::NoContribution {});
        }

        // Check if the total funds available in the cooperative are sufficient
        let total_available_funds = cooperative
            .total_funds
            .iter()
            .find(|(id, _)| *id == w_token_idx as u64)
            .map(|(_, amount)| *amount)
            .unwrap_or(Uint128::zero());

        if total_available_funds < amount {
            return Err(ContractError::InsufficientPoolFunds {});
        }

        // Calculate the member's share of rewards based on the current contribution
        let total_pooled_funds = total_available_funds;
        let member_share = if total_pooled_funds.is_zero() {
            Uint128::zero()
        } else {
            let rewards_pool_key = (cooperative_name.clone(), w_token_idx as u64);
            let rewards_pool = REWARDS_POOLS.load(deps.storage, rewards_pool_key)?;
            (amount * rewards_pool.total_rewards) / total_pooled_funds
        };

        // Check if the member has sufficient rewards
        if member_idx >= cooperative.members.len()
            || w_token_idx >= cooperative.members[member_idx].share.len()
        {
            return Err(ContractError::InsufficientRewards {});
        }

        let share = &mut cooperative.members.clone()[member_idx].share[w_token_idx];
        if share.1 < member_share {
            return Err(ContractError::InsufficientRewards {});
        }

        // Update contribution to zero
        cooperative.members[member_idx].contribution[contribution_idx].1 = Uint128::zero();

        // Update share to zero
        share.1 = Uint128::zero();

        // Update cooperative total funds
        let fund_idx = cooperative
            .total_funds
            .iter()
            .position(|(id, _)| *id == w_token_idx as u64)
            .ok_or(ContractError::InvalidToken {})?;

        cooperative.total_funds[fund_idx].1 =
            cooperative.total_funds[fund_idx].1.saturating_sub(amount);

        // Update the rewards pool's distributed rewards
        let rewards_pool_key = (cooperative_name.clone(), w_token_idx as u64);
        let mut rewards_pool = REWARDS_POOLS.load(deps.storage, rewards_pool_key.clone())?;
        rewards_pool.distributed_rewards += member_share;
        REWARDS_POOLS.save(deps.storage, rewards_pool_key, &rewards_pool)?;

        // Handle token transfer based on type
        let mut messages: Vec<CosmosMsg> = vec![];
        if w_token.is_native {
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
            let cw20_addr = w_token
                .contract_addr
                .clone()
                .ok_or(ContractError::InvalidToken {})?;

            // Create CW20 transfer message
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&cw20::Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount,
                })?,
                funds: vec![],
            }));
        }

        // Save updated cooperative data once
        COOPERATIVES.save(deps.storage, cooperative_name, &cooperative)?;

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "withdraw_contribution_and_rewards")
            .add_attribute("member", info.sender.to_string())
            .add_attribute("token", token.to_string())
            .add_attribute("amount", amount.to_string())
            .add_attribute("rewards", member_share.to_string()))
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
            voter: info.sender.clone(),
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
            .add_attribute("action", "vote")
            .add_attribute("cooperative_name", cooperative_name)
            .add_attribute("proposal_id", proposal_id.to_string()))
    }

    pub fn execute_withdraw_weight(
        deps: DepsMut,
        _env: Env,
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

        let msg: Vec<CosmosMsg> = vec![CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: state.weight_token,
                amount: weight,
            }],
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
            loans: vec![],
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
        QueryMsg::MemberContributionAndShare {
            cooperative_name,
            member_address,
        } => to_json_binary(&query::query_member_contribution_and_share(
            deps,
            cooperative_name,
            member_address.to_string(),
        )?),
        QueryMsg::GetProposal { proposal_id } => {
            to_json_binary(&query::get_proposal(deps, proposal_id)?)
        }
        QueryMsg::GetWhitelistedTokens { cooperative_name } => {
            to_json_binary(&query::get_whitelisted_tokens(deps, cooperative_name)?)
        }
        QueryMsg::ListCooperatives {} => to_json_binary(&query::list_cooperative(deps)?),
        QueryMsg::GetTokenId { token } => to_json_binary(&query::get_token_id(deps, token)?),
    }
}

pub mod query {
    use cosmwasm_std::StdError;

    use crate::{
        msg::{
            GetCooperativeResponse, GetListCooperativesResponse, GetMemberInfoResponse,
            GetProposalResponse, GetTokenIdResponse, GetWhitelistedTokensResponse,
            MemberContributionAndShareResponse, TokenAmount, TokenInfo,
        },
        state::WhitelistedTokenId,
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
                    loans: vec![],
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
                    loans: vec![],
                },
            });
        }

        let info = member_data.unwrap();
        Ok(GetMemberInfoResponse { info: info.clone() })
    }

    pub fn query_member_contribution_and_share(
        deps: Deps,
        cooperative_name: String,
        member_address: String,
    ) -> StdResult<MemberContributionAndShareResponse> {
        let cooperative_name = cooperative_name.trim().to_lowercase();
        let member_addr = deps.api.addr_validate(&member_address)?;

        // Get the cooperative
        let cooperative = COOPERATIVES.load(deps.storage, cooperative_name.clone())?;

        // Find the member in the cooperative
        let member = cooperative
            .members
            .iter()
            .find(|m| m.address == member_addr)
            .ok_or_else(|| {
                cosmwasm_std::StdError::generic_err(format!(
                    "Member {} not found in cooperative {}",
                    member_address, cooperative_name
                ))
            })?;

        // Collect token information for all tokens in the cooperative
        let mut token_info: Vec<TokenInfo> = Vec::new();
        for token in &cooperative.whitelisted_tokens {
            let token_id = if token.is_native {
                TOKENS.load(deps.storage, Addr::unchecked("NATIVE"))?
            } else {
                TOKENS.load(deps.storage, token.contract_addr.as_ref().unwrap().clone())?
            };

            token_info.push(TokenInfo {
                token_id,
                denom: token.denom.clone(),
                contract_addr: token.contract_addr.as_ref().map(|addr| addr.to_string()),
                is_native: token.is_native,
                symbol: None,
                name: Some(token.denom.clone()),
            });
        }

        // Convert member contributions to TokenAmount format
        let contributions: Vec<TokenAmount> = member
            .contribution
            .iter()
            .map(|(token_id, amount)| TokenAmount {
                token_id: *token_id,
                amount: *amount,
                symbol: None,
                name: None,
            })
            .collect();

        // Calculate up-to-date shares for each token the member has contributed to
        let mut calculated_shares: Vec<(WhitelistedTokenId, Uint128)> = Vec::new();

        for &(token_id, contribution_amount) in &member.contribution {
            // Get the total funds for this token in the cooperative (including accumulated interest)
            let total_token_funds = cooperative
                .total_funds
                .iter()
                .find(|(t_id, _)| t_id == &token_id)
                .map(|(_, amount)| *amount)
                .unwrap_or(Uint128::zero());

            // If there are no funds for this token, share is zero
            if total_token_funds.is_zero() {
                calculated_shares.push((token_id, Uint128::zero()));
                continue;
            }

            // Calculate the sum of all member contributions for this token
            let total_member_contributions_for_token: Uint128 = cooperative
                .members
                .iter()
                .flat_map(|m| m.contribution.iter())
                .filter_map(|&(t_id, amount)| if t_id == token_id { Some(amount) } else { None })
                .sum();

            // Calculate the member's percentage of contributions
            // Use Decimal for precise percentage calculation
            let contribution_percentage = if !total_member_contributions_for_token.is_zero() {
                Decimal::from_ratio(contribution_amount, total_member_contributions_for_token)
            } else {
                Decimal::zero()
            };

            // Calculate member's share of the total funds (including interest)
            // Share = (member's contribution percentage) * total cooperative funds for this token
            let share_amount = contribution_percentage.to_uint_ceil() * total_token_funds;

            calculated_shares.push((token_id, share_amount));
        }

        // Convert calculated shares to TokenAmount format
        let shares: Vec<TokenAmount> = calculated_shares
            .iter()
            .map(|(token_id, amount)| TokenAmount {
                token_id: *token_id,
                amount: *amount,
                symbol: None,
                name: None,
            })
            .collect();

        Ok(MemberContributionAndShareResponse {
            member_address,
            cooperative_name,
            contributions,
            shares,
            loans: member.loans.clone(),
            token_info,
        })
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

    pub fn list_cooperative(deps: Deps) -> StdResult<GetListCooperativesResponse> {
        let cooperatives = COOPERATIVES
            .keys(deps.storage, None, None, Order::Descending)
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
