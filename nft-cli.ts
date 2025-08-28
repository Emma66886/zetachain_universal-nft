#!/usr/bin/env node

import { Command } from "commander";
import { ethers } from "hardhat";

const program = new Command();

program
  .name("nft-cli")
  .description("Universal NFT CLI tool")
  .version("1.0.0");

// Mint NFT command
program
  .command("mint")
  .description("Mint a new Universal NFT")
  .requiredOption("-c, --contract <address>", "Contract address")
  .requiredOption("-t, --to <address>", "Recipient address")
  .requiredOption("-u, --uri <uri>", "Token URI/metadata URL")
  .option("-r, --rpc <url>", "RPC URL", "http://localhost:8545")
  .option("-k, --private-key <key>", "Private key")
  .action(async (options) => {
    try {
      const [signer] = await ethers.getSigners();
      console.log("🔑 Using account:", signer.address);

      const contract = await ethers.getContractAt("UniversalNFT", options.contract);
      const tx = await contract.safeMint(options.to, options.uri);
      const receipt = await tx.wait();

      console.log("✅ NFT minted successfully");
      console.log("📄 Transaction hash:", receipt?.hash);

      // Get token ID from Transfer event
      const transferEvent = receipt?.logs.find((log: any) => {
        try {
          const parsed = contract.interface.parseLog(log);
          return parsed?.name === "Transfer" && parsed?.args.from === ethers.ZeroAddress;
        } catch {
          return false;
        }
      });

      if (transferEvent) {
        const parsed = contract.interface.parseLog(transferEvent);
        console.log("🎨 Token ID:", parsed?.args.tokenId.toString());
      }
    } catch (error) {
      console.error("❌ Error:", error);
      process.exit(1);
    }
  });

// Transfer cross-chain command
program
  .command("transfer")
  .description("Transfer NFT cross-chain")
  .requiredOption("-c, --contract <address>", "Contract address")
  .requiredOption("-i, --token-id <id>", "Token ID")
  .requiredOption("-r, --receiver <address>", "Recipient address")
  .requiredOption("-d, --destination <address>", "Destination chain ZRC-20")
  .requiredOption("-a, --amount <amount>", "ZETA amount for gas")
  .option("--rpc <url>", "RPC URL", "http://localhost:8545")
  .option("-k, --private-key <key>", "Private key")
  .action(async (options) => {
    try {
      const [signer] = await ethers.getSigners();
      console.log("🔑 Using account:", signer.address);

      const contract = await ethers.getContractAt("UniversalNFT", options.contract);
      
      // Convert receiver to bytes32 if needed
      let receiver: string;
      if (options.receiver.length === 44) {
        // Solana address
        const bs58 = require("bs58");
        const pubkeyBytes = bs58.decode(options.receiver);
        receiver = "0x" + Buffer.from(pubkeyBytes).toString("hex").padStart(64, "0");
      } else {
        // Ethereum address
        receiver = "0x" + options.receiver.slice(2).padStart(64, "0");
      }

      const value = ethers.parseEther(options.amount);
      
      console.log("🚀 Transferring NFT cross-chain...");
      console.log("🎨 Token ID:", options.tokenId);
      console.log("📍 Destination:", options.destination);
      console.log("👤 Receiver:", options.receiver);

      const tx = await contract.transferCrossChain(
        options.tokenId,
        receiver,
        options.destination,
        { value }
      );

      const receipt = await tx.wait();
      console.log("✅ Cross-chain transfer initiated");
      console.log("📄 Transaction hash:", receipt?.hash);
    } catch (error) {
      console.error("❌ Error:", error);
      process.exit(1);
    }
  });

// Set connected command
program
  .command("set-connected")
  .description("Set connected contract for cross-chain transfers")
  .requiredOption("-c, --contract <address>", "UniversalNFT contract address")
  .requiredOption("-z, --zrc20 <address>", "ZRC-20 token address")
  .requiredOption("-a, --address <hex>", "Connected contract address (hex)")
  .option("--rpc <url>", "RPC URL", "http://localhost:8545")
  .option("-k, --private-key <key>", "Private key")
  .action(async (options) => {
    try {
      const [signer] = await ethers.getSigners();
      console.log("🔑 Using account:", signer.address);

      const contract = await ethers.getContractAt("UniversalNFT", options.contract);
      
      console.log("🔗 Setting connected chain...");
      const tx = await contract.setConnected(options.zrc20, options.address);
      const receipt = await tx.wait();

      console.log("✅ Connected chain set successfully");
      console.log("📄 Transaction hash:", receipt?.hash);
    } catch (error) {
      console.error("❌ Error:", error);
      process.exit(1);
    }
  });

// Info command
program
  .command("info")
  .description("Get NFT information")
  .requiredOption("-c, --contract <address>", "Contract address")
  .requiredOption("-i, --token-id <id>", "Token ID")
  .option("--rpc <url>", "RPC URL", "http://localhost:8545")
  .action(async (options) => {
    try {
      const contract = await ethers.getContractAt("UniversalNFT", options.contract);
      
      const owner = await contract.ownerOf(options.tokenId);
      const uri = await contract.tokenURI(options.tokenId);
      
      console.log("📊 NFT Information:");
      console.log("🎨 Token ID:", options.tokenId);
      console.log("👤 Owner:", owner);
      console.log("🔗 URI:", uri);
    } catch (error) {
      console.error("❌ NFT not found or error:", error);
      process.exit(1);
    }
  });

program.parse();
