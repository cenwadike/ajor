import { SigningCosmWasmClient, CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";

// Constants
const RPC_ENDPOINT = "https://rpc-palvus.pion-1.ntrn.tech"; // Neutron RPC endpoint
const CONTRACT_ADDRESS = "neutron1lf2ujetj5dcs4l874jeefdp9nngkquauk2qav5zvpmrzh63xs4rqyu7hcv"; 

// Main class for interacting with the cooperative contract
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
        token_addr: tokenAddr,
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
      const result = await this.client.execute(
        this.address,
        CONTRACT_ADDRESS,
        msg,
        "auto"
      );
      
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


  // Increase allowance for CW20 tokens
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

    return await this.client.execute(
        this.address,
        cw20TokenContract,
        msg,
        "auto",
        undefined,
        funds
    );
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

  /**
   * List all cooperatives
   */
  async listCooperatives(min = "", max = "") {
    if (!this.client) throw new Error("Client not initialized");
    
    const query = {
      list_cooperatives: {
        min,
        max,
      },
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


import dotenv from "dotenv"

dotenv.config()

// Example usage
const runExample = async () => {
  try {
      const mnemonic = process.env.MNEMONIC
      const client = new CooperativeClient(mnemonic);
      await client.connect();
      
      // Example: Create a cooperative
      const riskProfile = {
          interest_rate: "0.05", // 5%
          collateralization_ratio: "1.5", // 150%
      };
      
      const initialMembers = [
          {
            address: "neutron107nhk9pqhp446fr0fc83z0v82rg9guy8runkuz",
            contribution: [[0, "0"]], 
            share: [[0, "0"]], 
            joined_at: Math.floor(Date.now() / 1000), // Current timestamp in seconds
            reputation_score: "1.0",
            active_loans: [],
          }
      ];
      
      const initialWhitelistedTokens = [
          {
            denom: "untrn",
            is_native: true,
            max_loan_ratio: "0.7", // 70%
          },
          {
            denom: "tATOM",
            contract_addr: "neutron1sr60e2velepytzsdyuutcmccl9n2p2lu3pjcggllxyc9rzyu562sqegazj", // tATOM
            is_native: false,
            max_loan_ratio: "0.65", // 65%
          }
      ];
      
      const result = await client.createCooperative(
          "My Cooperative",
          riskProfile,
          initialMembers,
          initialWhitelistedTokens
      );
      
      console.log("Cooperative created successfully:", result);
      
      // Example: Fund a cooperative
      const fundNativeResult = await client.fundCooperative(
          "My Cooperative",
          "untrn",
          true,
          "1", 
          [
            {
                denom: "untrn",
                amount: "1"
            }
          ]
      );
      
      console.log("Cooperative funded with native successfully:", fundNativeResult);

      // increase tATOM allowance 
      let atomAllowanceRes = await client.increaseAllowance(
        "neutron1sr60e2velepytzsdyuutcmccl9n2p2lu3pjcggllxyc9rzyu562sqegazj", 
        CONTRACT_ADDRESS, 
        "1"
      );
      console.log("Increase Allowance Transaction Result:", atomAllowanceRes);

      const fundCw20Result = await client.fundCooperative(
          "My Cooperative",
          "neutron1sr60e2velepytzsdyuutcmccl9n2p2lu3pjcggllxyc9rzyu562sqegazj",
          false,
          "1",
          []
      );
    
      console.log("Cooperative funded with CW20 successfully:", fundCw20Result);
      
      // Example: Create a proposal to whitelist a new token
      const proposal = {
          id: 1, // This will be assigned by the contract
          description: "Add NGN as a whitelisted token",
          data: {
            denom: "tNGN", 
            is_native: false,
            token_addr: "neutron1he6zd5kk03cs5ywxk5tth9qfewxwnh7k9hjwekr7gs9gl9argadsqdc9rp", // tNGN
            max_loan_ratio: "0.6", // 60%
          },
          votes: [],
          aye_count: 0,
          nay_count: 0,
          aye_weights: 0,
          nay_weights: 0,
          end_time: Math.floor(Date.now() / 1000) + 604800, // 1 week from now
          proposal_type: "WhitelistToken",
          executed: false
      };
      
      const proposeResult = await client.propose(
          "My Cooperative",
          proposal
      );
      
      console.log("Proposal created successfully:", proposeResult);
    
  } catch (error) {
      console.error("Error in example:", error);
  }
};

runExample().catch(console.error);
