import { parseEther } from "ethers";
import { task } from "hardhat/config";
import type { HardhatRuntimeEnvironment } from "hardhat/types";
import { UniversalNFT } from "../../types";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const [signer] = await hre.ethers.getSigners();
  console.log("🔑 Using account:", signer.address);

  const factory = await hre.ethers.getContractFactory("UniversalNFT");
  const contract = factory.attach(args.contract) as UniversalNFT;

  // Convert receiver to bytes32 if it's a Solana address
  let receiver: string;
  if (args.receiver.length === 44) {
    // Assume it's a Solana address (base58), convert to bytes32
    const bs58 = require("bs58");
    const pubkeyBytes = bs58.decode(args.receiver);
    receiver = "0x" + Buffer.from(pubkeyBytes).toString("hex").padStart(64, "0");
  } else {
    // Convert Ethereum address to bytes32
    receiver = "0x" + args.receiver.slice(2).padStart(64, "0");
  }

  const value = parseEther(args.amount);
  
  console.log("🚀 Transferring NFT cross-chain...");
  console.log("🎨 Token ID:", args.tokenId);
  console.log("📍 Destination:", args.destination);
  console.log("👤 Receiver:", args.receiver);
  console.log("💰 Gas amount:", args.amount, "ZETA");

  const tx = await contract.transferCrossChain(
    args.tokenId,
    receiver,
    args.destination,
    { value }
  );

  const receipt = await tx.wait();

  console.log("✅ Cross-chain NFT transfer initiated");
  console.log("📄 Transaction hash:", receipt?.hash);

  if (receipt?.logs) {
    receipt.logs.forEach((log: any) => {
      try {
        const parsed = contract.interface.parseLog(log);
        console.log(`📝 Event: ${parsed?.name}`, parsed?.args);
      } catch {
        // Ignore unparseable logs
      }
    });
  }
};

task("transfer-nft-cross-chain", "Transfer Universal NFT to another chain")
  .addParam("contract", "Contract address")
  .addParam("tokenId", "Token ID to transfer")
  .addParam("receiver", "Recipient address on destination chain")
  .addParam("destination", "Destination chain ZRC-20 address")
  .addParam("amount", "Amount of ZETA for gas fees")
  .addOptionalParam("rpc", "RPC URL", "http://localhost:8545")
  .addOptionalParam("privateKey", "Private key")
  .setAction(main);
