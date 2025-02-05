use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cooperative with this name already exists")]
    CooperativeAlreadyExists {},

    #[error("Cooperative not found")]
    CooperativeNotFound {},

    #[error("Already a Member")]
    AlreadyMember {},

    #[error("Member not found")]
    MemberNotFound {},

    #[error("Member data not found")]
    MemberDataNotFound {},

    #[error("No Funds")]
    NoFunds {},

    #[error("Fund must match amount")]
    InvalidFundAmount {},

    #[error("Invalid token")]
    InvalidToken {},

    #[error("Max whitelisted tokens reached")]
    MaxWhitelistedTokensReached {},

    #[error("Token already whitelisted")]
    TokenAlreadyWhitelisted {},

    #[error("Insufficient funds")]
    InsufficientFunds {},

    #[error("Insufficient collateral")]
    InsufficientCollateral {},

    #[error("No active loan")]
    NoActiveLoan {},

    #[error("Loan ratio exceeded")]
    LoanRatioExceeded {},

    #[error("Invalid proposal")]
    InvalidProposal {},

    #[error("Proposal already ended")]
    ProposalEnded {},

    #[error("Already voted")]
    AlreadyVoted {},

    #[error("Proposal was rejected")]
    ProposalRejected {},

    #[error("Proposal is in process")]
    ProposalInProcess {},

    #[error("Proposal not found")]
    ProposalNotFound {},

    #[error("Proposal already executed")]
    ProposalAlreadyExecuted {},

    #[error("No weight to withdraw")]
    NoWeightsToWithdraw {},

    #[error("No reward available")]
    NoRewardsAvailable {},

    #[error("Insufficient reward")]
    InsufficientRewards {},

    #[error("Insufficient pool funds")]
    InsufficientPoolFunds {},

    #[error("Feature not implemented")]
    NotImplemented {},
}
