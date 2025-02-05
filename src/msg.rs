use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::state::{Cooperative, CorporativeName, Member, Proposal, RiskProfile, WhitelistedToken};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateTokenPrice {
        token_addr: Addr,
        usd_price: u128,
    },
    CreateCooperative {
        name: String,
        risk_profile: RiskProfile,
        initial_members: Vec<Member>,
        initial_whitelisted_tokens: Vec<WhitelistedToken>,
    },
    FundCooperative {
        cooperative_name: CorporativeName,
        token: String,
        amount: Uint128,
    },
    Borrow {
        cooperative_name: CorporativeName,
        tokens_in: Vec<Addr>,
        amount_in: Vec<Uint128>,
        token_out: Addr,
        min_amount_out: Uint128,
    },
    Repay {
        cooperative_name: CorporativeName,
        token: Addr,
    },
    Propose {
        cooperative_name: CorporativeName,
        proposal: Proposal,
    },
    Vote {
        cooperative_name: CorporativeName,
        proposal_id: u64,
        weight: Uint128,
        aye: bool,
    },
    WithdrawWeight {
        cooperative_name: CorporativeName,
        proposal_id: u64,
    },
    WithdrawContributionAndReward {
        cooperative_name: CorporativeName,
        token: Addr,
    },
    ExecuteProposal {
        cooperative_name: CorporativeName,
        proposal_id: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetCooperativeResponse)]
    GetCooperative { cooperative_name: CorporativeName },

    #[returns(GetMemberInfoResponse)]
    GetMemberInfo {
        cooperative_name: CorporativeName,
        member: Addr,
    },

    #[returns(GetListCooperativesResponse)]
    ListCooperatives { min: String, max: String },

    #[returns(GetProposalResponse)]
    GetProposal { proposal_id: u64 },

    #[returns(GetWhitelistedTokensResponse)]
    GetWhitelistedTokens { cooperative_name: CorporativeName },
}

#[cw_serde]
pub struct GetCooperativeResponse {
    pub corporative: Cooperative,
}

#[cw_serde]
pub struct GetMemberInfoResponse {
    pub info: Member,
}

#[cw_serde]
pub struct GetListCooperativesResponse {
    pub cooperatives: Vec<CorporativeName>,
}

#[cw_serde]
pub struct GetProposalResponse {
    pub proposal: Proposal,
}

#[cw_serde]
pub struct GetWhitelistedTokensResponse {
    pub tokens: Vec<WhitelistedToken>,
}
