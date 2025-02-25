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
- `loans`: List of loans taken by the member.

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

Here's how to call contract functions using JavaScript:

```ts
    class CooperativeClient {
        constructor(mnemonic) {
            this.mnemonic = mnemonic;
            this.client = null;
            this.wallet = null;
            this.address = null;
        }

        /**
         * Initialize the client with a mnemonic
         */
        async connect() {
            try {
                // Create a wallet from mnemonic
                this.wallet = await DirectSecp256k1HdWallet.fromMnemonic(this.mnemonic, {
                    prefix: "neutron", // Neutron address prefix
                });

                // Get the first account from the wallet
                const accounts = await this.wallet.getAccounts();
                this.address = accounts[0].address;

                // Create a signing client
                this.client = await SigningCosmWasmClient.connectWithSigner(
                    RPC_ENDPOINT,
                    this.wallet,
                    { gasPrice: GasPrice.fromString("0.025untrn") } // Neutron gas denom
                );

                console.log(`Connected with address: ${this.address}`);
            } catch (error) {
                console.error("Failed to connect:", error);
                throw error;
            }
        }

        /**
         * Update token price
         */
        async updateTokenPrice(tokenAddr, usdPrice) {
            if (!this.client || !this.address) throw new Error("Client not initialized");
            
            const msg = {
                update_token_price: {
                    token_addr: tokenAddr.toString(),
                    usd_price: usdPrice.toString(),
                },
            };

            try {
                const result = await this.client.execute(
                    this.address,
                    CONTRACT_ADDRESS,
                    msg,
                    "auto"
                );
                
                console.log("Token price updated:", result);
                return result;
            } catch (error) {
                console.error("Failed to update token price:", error);
                throw error;
            }
        }

        /**
         * Create a new cooperative
         */
        async createCooperative(
            name,
            riskProfile,
            initialMembers,
            initialWhitelistedTokens
        ) {
            if (!this.client || !this.address) throw new Error("Client not initialized");
            
            const msg = {
                create_cooperative: {
                    name,
                    risk_profile: riskProfile,
                    initial_members: initialMembers,
                    initial_whitelisted_tokens: initialWhitelistedTokens,
                },
            };

            try {
                const result = await this.client.execute(
                    this.address,
                    CONTRACT_ADDRESS,
                    msg,
                    "auto"
                );
                
                console.log("Cooperative created:", result);
                return result;
            } catch (error) {
                console.error("Failed to create cooperative:", error);
                throw error;
            }
        }

        /**
         * Fund a cooperative
         */
        async fundCooperative(
            cooperativeName,
            token,
            is_native,
            amount,
            funds
        ) {
            if (!this.client || !this.address) throw new Error("Client not initialized");
            
            const msg = {
                fund_cooperative: {
                    cooperative_name: cooperativeName,
                    token,
                    is_native,
                    amount,
                },
            };

            try {
                const result = await this.client.execute(
                    this.address,
                    CONTRACT_ADDRESS,
                    msg,
                    "auto",
                    undefined,
                    funds
                );
                
                console.log("Cooperative funded:", result);
                return result;
            } catch (error) {
                console.error("Failed to fund cooperative:", error);
                throw error;
            }
        }

        /**
         * Borrow tokens from a cooperative
         */
        async borrow(
            cooperativeName,
            tokensIn,
            amountIn,
            tokenOut,
            minAmountOut,
            funds
        ) {
            if (!this.client || !this.address) throw new Error("Client not initialized");
            
            const msg = {
                borrow: {
                    cooperative_name: cooperativeName,
                    tokens_in: tokensIn,
                    amount_in: amountIn,
                    token_out: tokenOut,
                    min_amount_out: minAmountOut,
                },
            };

            try {
                const result = await this.client.execute(
                    this.address,
                    CONTRACT_ADDRESS,
                    msg,
                    "auto",
                    undefined,
                    funds
                );
                
                console.log("Borrow executed:", result);
                return result;
            } catch (error) {
                console.error("Failed to borrow:", error);
                throw error;
            }
        }

        /**
         * Repay a loan
         */
        async repay(
            cooperativeName,
            token,
            funds
        ) {
            if (!this.client || !this.address) throw new Error("Client not initialized");
            
            const msg = {
                repay: {
                    cooperative_name: cooperativeName,
                    token,
                },
            };

            try {
                const result = await this.client.execute(
                    this.address,
                    CONTRACT_ADDRESS,
                    msg,
                    "auto",
                    undefined,
                    funds
                );
            
                console.log("Loan repaid:", result);
                return result;
            } catch (error) {
                console.error("Failed to repay loan:", error);
                throw error;
            }
        }

        /**
         * Create a new proposal
         */
        async propose(
            cooperativeName,
            proposal
        ) {
            if (!this.client || !this.address) throw new Error("Client not initialized");
            
            const msg = {
                propose: {
                    cooperative_name: cooperativeName,
                    proposal,
                },
            };

            try {
                const result = await this.client.execute(
                    this.address,
                    CONTRACT_ADDRESS,
                    msg,
                    "auto"
                );
                
                console.log("Proposal created:", result);
                return result;
            } catch (error) {
                console.error("Failed to create proposal:", error);
                throw error;
            }
        }

        /**
         * Vote on a proposal
         */
        async vote(
            cooperativeName,
            proposalId,
            weight,
            aye
        ) {
            if (!this.client || !this.address) throw new Error("Client not initialized");
            
            const msg = {
                vote: {
                    cooperative_name: cooperativeName,
                    proposal_id: proposalId,
                    weight,
                    aye,
                },
            };

            try {
                let result;
                if (weight > 0) {
                    result = await this.client.execute(
                        this.address,
                        CONTRACT_ADDRESS,
                        msg,
                        "auto",
                        "",
                        [
                            {
                            denom: "untrn",
                            amount: weight
                            }
                        ]
                    );
                } else {
                    result = await this.client.execute(
                        this.address,
                        CONTRACT_ADDRESS,
                        msg,
                        "auto",
                    );        
                }
                
                console.log("Vote cast:", result);
                return result;
            } catch (error) {
                console.error("Failed to vote:", error);
                throw error;
            }
        }

        /**
         * Withdraw voting weight from a proposal
         */
        async withdrawWeight(
            cooperativeName,
            proposalId
        ) {
            if (!this.client || !this.address) throw new Error("Client not initialized");
            
            const msg = {
                withdraw_weight: {
                    cooperative_name: cooperativeName,
                    proposal_id: proposalId,
                },
            };

            try {
                const result = await this.client.execute(
                    this.address,
                    CONTRACT_ADDRESS,
                    msg,
                    "auto"
                );
                
                console.log("Weight withdrawn:", result);
                return result;
            } catch (error) {
                console.error("Failed to withdraw weight:", error);
                throw error;
            }
        }

        /**
         * Withdraw contribution and reward from a cooperative
         */
        async withdrawContributionAndReward(
            cooperativeName,
            token
        ) {
            if (!this.client || !this.address) throw new Error("Client not initialized");
            
            const msg = {
                withdraw_contribution_and_reward: {
                    cooperative_name: cooperativeName,
                    token,
                },
            };

            try {
                const result = await this.client.execute(
                    this.address,
                    CONTRACT_ADDRESS,
                    msg,
                    "auto"
                );
                
                console.log("Contribution and reward withdrawn:", result);
                return result;
            } catch (error) {
                console.error("Failed to withdraw contribution and reward:", error);
                throw error;
            }
        }

        /**
         * Execute a proposal
         */
        async executeProposal(
            cooperativeName,
            proposalId
        ) {
            if (!this.client || !this.address) throw new Error("Client not initialized");
            
            const msg = {
                execute_proposal: {
                    cooperative_name: cooperativeName,
                    proposal_id: proposalId,
                },
            };

            try {
                const result = await this.client.execute(
                    this.address,
                    CONTRACT_ADDRESS,
                    msg,
                    "auto"
                );
                
                console.log("Proposal executed:", result);
                return result;
            } catch (error) {
                console.error("Failed to execute proposal:", error);
                throw error;
            }
        }

        /**
         * Increase allowance for CW20 tokens
         */
        async increaseAllowance(
            cw20TokenContract,
            spender,
            amount,
            funds,
        ) {
            if (!this.client || !this.address) throw new Error("Client not initialized");

            const msg = {
                increase_allowance: {
                    spender: spender,
                    amount: amount
                }
            };

            try {
                return await this.client.execute(
                    this.address,
                    cw20TokenContract,
                    msg,
                    "auto",
                    undefined,
                    funds
                );
            } catch (error) {
                console.error("Failed to increase allowance")
            }
        }


        // Query methods

        /**
         * Get cooperative information
         */
        async getCooperative(cooperativeName) {
            if (!this.client) throw new Error("Client not initialized");
            
            const query = {
                get_cooperative: {
                    cooperative_name: cooperativeName,
                },
            };

            try {
                const result = await this.client.queryContractSmart(CONTRACT_ADDRESS, query);
                return result;
            } catch (error) {
                console.error("Failed to get cooperative:", error);
                throw error;
            }
        }

        /**
         * Get member information
         */
        async getMemberInfo(cooperativeName, member) {
            if (!this.client) throw new Error("Client not initialized");
            
            const query = {
                get_member_info: {
                    cooperative_name: cooperativeName,
                    member,
                },
            };

            try {
                const result = await this.client.queryContractSmart(CONTRACT_ADDRESS, query);
                return result;
            } catch (error) {
                console.error("Failed to get member info:", error);
                throw error;
            }
        }

        async getMemberInfoAndShare(cooperativeName, memberAddress) {
            if (!this.client) throw new Error("Client not initialized");
            
            const query = {
                member_contribution_and_share: {
                    cooperative_name: cooperativeName,
                    member_address: memberAddress,
                },
            };

            try {
                const result = await this.client.queryContractSmart(CONTRACT_ADDRESS, query);
                return result;
            } catch (error) {
                console.error("Failed to get member contribution and shares:", error);
                throw error;
            }
        }

        /**
         * List all cooperatives
         */
        async listCooperatives() {
            if (!this.client) throw new Error("Client not initialized");
            
            const query = {
                list_cooperatives: {},
            };

            try {
                const result = await this.client.queryContractSmart(CONTRACT_ADDRESS, query);
                return result;
            } catch (error) {
                console.error("Failed to list cooperatives:", error);
                throw error;
            }
        }

        /**
         * Get proposal information
         */
        async getProposal(proposalId) {
            if (!this.client) throw new Error("Client not initialized");
            
            const query = {
                get_proposal: {
                    proposal_id: proposalId,
                },
            };

            try {
                const result = await this.client.queryContractSmart(CONTRACT_ADDRESS, query);
                return result;
            } catch (error) {
                console.error("Failed to get proposal:", error);
                throw error;
            }
        }

        /**
         * Get whitelisted tokens for a cooperative
         */
        async getWhitelistedTokens(cooperativeName) {
            if (!this.client) throw new Error("Client not initialized");
            
            const query = {
                get_whitelisted_tokens: {
                    cooperative_name: cooperativeName,
                },
            };

            try {
                const result = await this.client.queryContractSmart(CONTRACT_ADDRESS, query);
                return result;
            } catch (error) {
                console.error("Failed to get whitelisted tokens:", error);
                throw error;
            }
        }

        /**
         * Get whitelisted tokens id
         */
        async getTokenId(token) {
            if (!this.client) throw new Error("Client not initialized");
            
            const query = {
                get_token_id: {
                    token,
                },
            };

            try {
                const result = await this.client.queryContractSmart(CONTRACT_ADDRESS, query);
                return result;
            } catch (error) {
                console.error("Failed to get token id:", error);
                throw error;
            }
        }   
    }

    export default {
        CooperativeClient
    };
```