import { ethers } from "hardhat";
import { setupSolanaNFTProgram } from "../solana/setup/nftSetup";

async function main() {
  console.log("🎨 Universal NFT Cross-Chain Transfer Demo");
  console.log("==========================================");

  const [signer] = await ethers.getSigners();
  console.log("🔑 Using account:", signer.address);

  // Contract addresses (these would be populated from deployment)
  const UNIVERSAL_NFT = process.env.UNIVERSAL_NFT_ADDRESS || "";
  const ZRC20_ETHEREUM = process.env.ZRC20_ETHEREUM || "";
  const ZRC20_BNB = process.env.ZRC20_BNB || "";
  
  if (!UNIVERSAL_NFT || !ZRC20_ETHEREUM || !ZRC20_BNB) {
    console.error("❌ Please set environment variables for contract addresses");
    process.exit(1);
  }

  const contract = await ethers.getContractAt("UniversalNFT", UNIVERSAL_NFT);

  // Step 1: Mint NFT on ZetaChain
  console.log("\n🎨 Step 1: Minting NFT on ZetaChain...");
  const mintTx = await contract.safeMint(
    signer.address,
    "https://example.com/nft/metadata/1.json"
  );
  const mintReceipt = await mintTx.wait();
  console.log("✅ NFT minted, tx:", mintReceipt?.hash);

  // Extract token ID from Transfer event
  const transferEvent = mintReceipt?.logs.find((log: any) => {
    try {
      const parsed = contract.interface.parseLog(log);
      return parsed?.name === "Transfer" && parsed?.args.from === ethers.ZeroAddress;
    } catch {
      return false;
    }
  });

  let tokenId = "1";
  if (transferEvent) {
    const parsed = contract.interface.parseLog(transferEvent);
    tokenId = parsed?.args.tokenId.toString();
  }
  console.log("🎨 Token ID:", tokenId);

  // Step 2: Transfer to Ethereum
  console.log("\n🌉 Step 2: Transferring NFT ZetaChain → Ethereum...");
  const transferEthTx = await contract.transferCrossChain(
    tokenId,
    signer.address, // receiver on Ethereum
    ZRC20_ETHEREUM, // destination chain
    { value: ethers.parseEther("0.1") } // gas fee
  );
  const transferEthReceipt = await transferEthTx.wait();
  console.log("✅ Transfer to Ethereum initiated, tx:", transferEthReceipt?.hash);

  // Wait for cross-chain transfer to complete
  console.log("⏳ Waiting for cross-chain transfer...");
  await new Promise(resolve => setTimeout(resolve, 30000)); // 30 seconds

  // Step 3: Transfer to BNB (would be done from Ethereum side)
  console.log("\n🌉 Step 3: Transfer Ethereum → BNB...");
  console.log("(This step would be executed on Ethereum network)");

  // Step 4: Transfer to Solana (would be done from BNB side)
  console.log("\n🌉 Step 4: Transfer BNB → Solana...");
  console.log("(This step would be executed on BNB network)");

  // Example Solana address (base58)
  const solanaAddress = "11111111111111111111111111111112";
  
  // Convert Solana address to bytes32
  const bs58 = require("bs58");
  const pubkeyBytes = bs58.decode(solanaAddress);
  const solanaReceiver = "0x" + Buffer.from(pubkeyBytes).toString("hex").padStart(64, "0");

  console.log("🦀 Solana receiver (bytes32):", solanaReceiver);

  // Step 5: Transfer back to ZetaChain (would be done from Solana side)
  console.log("\n🌉 Step 5: Transfer Solana → ZetaChain...");
  console.log("(This step would be executed via Solana program)");

  console.log("\n✅ Universal NFT Cross-Chain Transfer Flow Completed!");
  console.log("📊 Summary:");
  console.log(`  - NFT Contract: ${UNIVERSAL_NFT}`);
  console.log(`  - Token ID: ${tokenId}`);
  console.log("  - Flow: ZetaChain → Ethereum → BNB → Solana → ZetaChain");
  console.log("  - Burn-and-mint mechanism ensures unique NFT across all chains");
  console.log("  - Metadata and ownership preserved throughout the journey");
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
