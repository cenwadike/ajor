# Corporative Lending System (Ajor)

This documentation provides an overview of the Corporative Lending System, explaining what it does, the structure of the code, and how to use it.

## Overview

The Corporative Lending System allows for the creation and management of cooperatives, where members can contribute funds, borrow loans, propose actions, and vote on proposals. The system also manages rewards generated from interest rates on loans, ensuring fair distribution of rewards to members based on their contributions.

## Structures

### State
Holds the overall state of the protocol:
- `owner`: The owner of the contract.
- `weight_token`: The weight token used in the protocol.
- `total_corporatives`: Total number of cooperatives.
- `total_pooled_funds`: Total pooled funds for each whitelisted token.
- `current_proposal_id`, `current_whitelisted_token_id`, `current_loan_id`: Counters for proposals, tokens, and loans.

### WhitelistedToken
Represents a token that is approved for use within the cooperative:
- `denom`: Token denomination.
- `contract_addr`: Optional contract address for CW20 tokens.
- `is_native`: Boolean indicating if the token is a native token.
- `max_loan_ratio`: Maximum loan-to-value ratio for the token.

### Cooperative
Represents a cooperative:
- `name`: Name of the cooperative.
- `total_funds`: Total funds pooled in the cooperative for each token.
- `members`: List of members in the cooperative.
- `risk_profile`: Risk profile of the cooperative.
- `whitelisted_tokens`: List of whitelisted tokens for the cooperative.

### Member
Represents a member of a cooperative:
- `address`: Address of the member.
- `contribution`: Contributions made by the member for each token.
- `share`: Shares of the rewards for each token.
- `joined_at`: Timestamp when the member joined.
- `reputation_score`: Reputation score of the member.
- `active_loans`: List of active loans taken by the member.

### Loan
Represents a loan:
- `id`: Loan ID.
- `amount`: Loan amount.
- `token`: Token address.
- `collaterals`: List of collateral addresses.
- `collaterals_amount`: List of collateral amounts.
- `interest_rate`: Interest rate on the loan.
- `status`: Status of the loan (Active, Repaid, Defaulted).

### CooperativeRewardsPool
Represents the rewards pool for each cooperative and token:
- `cooperative_name`: Name of the cooperative.
- `token_id`: Token ID.
- `total_rewards`: Total rewards generated.
- `distributed_rewards`: Total rewards distributed.

### MemberRewards
Tracks the last withdrawn rewards for each member:
- `cooperative_name`: Name of the cooperative.
- `member_address`: Address of the member.
- `token_id`: Token ID.
- `last_withdrawn_rewards`: Last withdrawn rewards.

## Functions

### InstantiateMsg
Defines the initial state of the contract. No parameters required.

### ExecuteMsg
Defines the executable functions of the contract:

- `UpdateTokenPrice`: Updates the price of a whitelisted token.
- `CreateCooperative`: Creates a new cooperative with initial members and tokens.
- `FundCooperative`: Contributes funds to a cooperative.
- `Borrow`: Initiates a loan from the cooperative.
- `Repay`: Repays an existing loan.
- `Propose`: Creates a new proposal for the cooperative.
- `Vote`: Casts a vote on a proposal.
- `WithdrawWeight`: Withdraws voting weight from a proposal.
- `WithdrawContributionAndReward`: Withdraws contribution and rewards from the cooperative.
- `ExecuteProposal`: Executes an approved proposal.

### QueryMsg
Defines the queryable functions of the contract:

- `GetCooperative`: Retrieves information about a specific cooperative.
- `GetMemberInfo`: Retrieves information about a specific member.
- `ListCooperatives`: Lists all cooperatives.
- `GetProposal`: Retrieves information about a specific proposal.
- `GetWhitelistedTokens`: Lists all whitelisted tokens for a cooperative.


## Run project

### Install Rust and Cargo

### Build project
    ```sh
        cargo wasm
    ```

### Test project 
    ```sh
        cargo test
    ``` 

## TypeScript Example

### Calling Contract Functions

Here's how to call contract functions using TypeScript:

        import { MsgExecuteContract, MsgQueryContract } from "cosmwasm";

        // Example: Updating Token Price
        const updateTokenPriceMsg = new MsgExecuteContract({
        sender: "<sender-address>",
        contract: "<contract-address>",
        msg: {
            UpdateTokenPrice: {
            token_addr: "<token-address>",
            usd_price: 1000, // Example price in USD
            }
        },
        funds: []
        });

        // Example: Creating a Cooperative
        const createCooperativeMsg = new MsgExecuteContract({
        sender: "<sender-address>",
        contract: "<contract-address>",
        msg: {
            CreateCooperative: {
            name: "My Cooperative",
            risk_profile: {
                interest_rate: "0.05",
                default_probability: "0.01",
                collateralization_ratio: "1.5",
            },
            initial_members: [
                {
                address: "<member-address>",
                contribution: [[1, 1000]],
                share: [[1, 100]],
                joined_at: 1610000000,// use blockchain time
                reputation_score: "1.0",
                active_loans: [],
                },
            ],
            initial_whitelisted_tokens: [
                {
                denom: "token-denom",
                contract_addr: null,
                is_native: true,
                max_loan_ratio: "0.7",
                },
            ],
            }
        },
        funds: []
        });

        // Example: Funding a Cooperative
        const fundCooperativeMsg = new MsgExecuteContract({
        sender: "<sender-address>",
        contract: "<contract-address>",
        msg: {
            FundCooperative: {
            cooperative_name: "My Cooperative",
            token: "token-denom",
            amount: 500,
            }
        },
        funds: []
        });

        // Example: Withdrawing Contribution and Reward
        const withdrawContributionAndRewardMsg = new MsgExecuteContract({
        sender: "<sender-address>",
        contract: "<contract-address>",
        msg: {
            WithdrawContributionAndReward: {
            cooperative_name: "My Cooperative",
            token: "token-address",
            }
        },
        funds: []
        });

        // Example: Querying Cooperative Information
        const getCooperativeQuery = new MsgQueryContract({
        contract: "<contract-address>",
        msg: {
            GetCooperative: {
            cooperative_name: "My Cooperative",
            }
        }
    });