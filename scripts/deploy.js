import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { readFileSync } from "fs";

import dotenv from "dotenv"

dotenv.config()

const rpcEndpoint = "https://rpc-palvus.pion-1.ntrn.tech";
const mnemonic = process.env.MNEMONIC;
const wasmFilePath = "../artifacts/ajor.wasm";

async function main() {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: "neutron",
  });

  const [firstAccount] = await wallet.getAccounts();

  const client = await SigningCosmWasmClient.connectWithSigner(
    rpcEndpoint,
    wallet,
    {
      gasPrice: GasPrice.fromString("0.025untrn"),
    }
  );

  const wasmCode = readFileSync(wasmFilePath);
  const uploadReceipt = await client.upload(firstAccount.address, wasmCode, "auto");
  console.log("Upload successful, code ID:", uploadReceipt.codeId);

  const initMsg = {};

  const instantiateReceipt = await client.instantiate(
    firstAccount.address, 
    uploadReceipt.codeId, 
    initMsg, 
    "Lend Pro (Ajor) Smart Contract", 
    "auto"
  );
  console.log("Contract instantiated at:", instantiateReceipt.contractAddress);
  // Upload successful, code ID: 10922
  // Contract instantiated at: neutron16qhawx7cy6cmte2jluu39d6j09emzml5yvmhdglyz0re99v6wpms0rh63m
}

main().catch(console.error);