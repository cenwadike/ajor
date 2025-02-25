use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};

use crate::state::{
    Cooperative, CorporativeName, Loan, Member, Proposal, RiskProfile, WhitelistedToken,
    WhitelistedTokenId,
};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateTokenPrice {
        token_addr: Addr,
        usd_price: Decimal,
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
        is_native: bool,
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

    #[returns(GetMemberInfoResponse)]
    MemberContributionAndShare {
        cooperative_name: CorporativeName,
        member_address: Addr,
    },

    #[returns(GetListCooperativesResponse)]
    ListCooperatives {},

    #[returns(GetProposalResponse)]
    GetProposal { proposal_id: u64 },

    #[returns(GetWhitelistedTokensResponse)]
    GetWhitelistedTokens { cooperative_name: CorporativeName },

    #[returns(GetTokenIdResponse)]
    GetTokenId { token: String },
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

#[cw_serde]
pub struct GetTokenIdResponse {
    pub token_id: WhitelistedTokenId,
}

// Member contribution and share response type
#[cw_serde]
pub struct MemberContributionAndShareResponse {
    /// The address of the member
    pub member_address: String,

    /// The name of the cooperative
    pub cooperative_name: String,

    /// The member's contributions to the cooperative
    /// Contains (token_id, amount) pairs
    pub contributions: Vec<TokenAmount>,

    /// The member's shares in the cooperative
    /// Contains (token_id, amount) pairs
    pub shares: Vec<TokenAmount>,

    /// The active loans of the member
    pub loans: Vec<Loan>,

    /// Token information for easy display
    pub token_info: Vec<TokenInfo>,
}

#[cw_serde]
pub struct TokenAmount {
    /// ID of the token
    pub token_id: u64,

    /// Amount of the token
    pub amount: Uint128,

    /// Symbol of the token (if available)
    pub symbol: Option<String>,

    /// Name of the token (if available)
    pub name: Option<String>,
}

#[cw_serde]
pub struct TokenInfo {
    /// ID of the token
    pub token_id: u64,

    /// Denom of the token (for native tokens)
    pub denom: String,

    /// Contract address (for CW20 tokens)
    pub contract_addr: Option<String>,

    /// Whether the token is native
    pub is_native: bool,

    /// Symbol of the token (if available)
    pub symbol: Option<String>,

    /// Name of the token (if available)
    pub name: Option<String>,
}
