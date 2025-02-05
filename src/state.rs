use std::fmt;

use schemars::JsonSchema;

use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

pub type WhitelistedTokenId = u64;
pub type CorporativeName = String;
pub type ProposalId = u64;

// Protocol metrics
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub weight_token: String,
    pub total_corporatives: u64,
    pub total_pooled_funds: Vec<(WhitelistedTokenId, Uint128)>,
    pub current_proposal_id: u64,
    pub current_whitelisted_token_id: u64,
    pub current_loan_id: u64,
}

// Whitelist token structure
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct WhitelistedToken {
    pub denom: String,
    pub contract_addr: Option<Addr>,
    pub is_native: bool,
    pub max_loan_ratio: Decimal,
}

// Corporative data
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Cooperative {
    pub name: CorporativeName,
    pub total_funds: Vec<(WhitelistedTokenId, Uint128)>,
    pub members: Vec<Member>,
    pub risk_profile: RiskProfile,
    pub whitelisted_tokens: Vec<WhitelistedToken>,
}

// Cooperative member data
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Member {
    pub address: Addr,
    pub contribution: Vec<(WhitelistedTokenId, Uint128)>,
    pub share: Vec<(WhitelistedTokenId, Uint128)>,
    pub joined_at: u64,
    pub reputation_score: Decimal,
    pub active_loans: Vec<Loan>, // User loans
}

// Cooperative risk profile
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct RiskProfile {
    pub interest_rate: Decimal,
    pub collateralization_ratio: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Loan {
    pub id: u64,
    pub amount: Uint128,
    pub token: Addr,
    pub collaterals: Vec<Addr>,
    pub collaterals_amount: Vec<Uint128>,
    pub interest_rate: Decimal,
    pub status: LoanStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum LoanStatus {
    Active,
    Repaid,
    Defaulted,
}

// Externally controlled liquidity
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LiquidityPosition {
    pub protocol: Addr,
    pub amount: Uint128,
    pub rewards: Uint128,
}

// Governance data
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Proposal {
    pub id: ProposalId,
    pub description: String,
    pub data: ProposalData,
    pub votes: Vec<Vote>,
    pub aye_count: u64,
    pub nay_count: u64,
    pub aye_weights: u64,
    pub nay_weights: u64,
    pub end_time: u64,
    pub quorum: Option<Decimal>,
    pub proposal_type: ProposalType,
    pub outcome: Option<ProposalOutcome>,
    pub executed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Vote {
    pub voter: Addr,
    pub conviction: Uint128,
    pub voted_at: u64,
}

// Whitelist token structure
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ProposalData {
    // whitelist data
    pub denom: Option<String>,
    pub token_addr: Option<Addr>,
    pub is_native: Option<bool>,
    pub max_loan_ratio: Option<Decimal>,

    // add member data
    pub new_member_addr: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum ProposalType {
    WhitelistToken,
    AddMember,
    AddLP,
    ApproveLoan,
    LiquidateCollateral,
}

impl fmt::Display for ProposalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum ProposalOutcome {
    Passed,
    Rejected,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Price {
    pub latest_price_to_usd: Uint128,
    pub last_updated_at: Timestamp,
}

// Rewards pool for each cooperative and each token
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CooperativeRewardsPool {
    pub cooperative_name: String,
    pub token_id: WhitelistedTokenId,
    pub total_rewards: Uint128,
    pub distributed_rewards: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MemberRewards {
    pub cooperative_name: String,
    pub member_address: Addr,
    pub token_id: WhitelistedTokenId,
    pub last_withdrawn_rewards: Uint128,
}

pub const STATE: Item<State> = Item::new("state");

pub const TOKENS: Map<Addr, WhitelistedTokenId> = Map::new("tokens");

pub const PRICES: Map<WhitelistedTokenId, Price> = Map::new("prices");

pub const COOPERATIVES: Map<CorporativeName, Cooperative> = Map::new("cooperatives");

pub const MEMBERS: Map<Addr, Vec<CorporativeName>> = Map::new("members");

pub const PROPOSALS: Map<ProposalId, Proposal> = Map::new("proposals");

pub const COOPERATIVES_PROPOSALS: Map<CorporativeName, Vec<ProposalId>> =
    Map::new("cooperatives_proposals");

pub const REWARDS_POOLS: Map<(CorporativeName, WhitelistedTokenId), CooperativeRewardsPool> =
    Map::new("rewards_pools");
