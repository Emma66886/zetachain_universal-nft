import { parseEther } from "ethers";
import { task } from "hardhat/config";
import type { HardhatRuntimeEnvironment } from "hardhat/types";
import { UniversalNFT } from "../types";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const [signer] = await hre.ethers.getSigners();
  console.log("🔑 Using account:", signer.address);

  const factory = await hre.ethers.getContractFactory("UniversalNFT");
  const contract = factory.attach(args.contract) as UniversalNFT;

  const tx = await contract.safeMint(args.to, args.uri);
  const receipt = await tx.wait();

  console.log("✅ NFT minted successfully");
  console.log("📄 Transaction hash:", receipt?.hash);
  
  // Get the minted token ID from the event
  const mintEvent = receipt?.logs.find((log: any) => {
    try {
      const parsed = contract.interface.parseLog(log);
      return parsed?.name === "Transfer";
    } catch {
      return false;
    }
  });

  if (mintEvent) {
    const parsed = contract.interface.parseLog(mintEvent);
    console.log("🎨 Token ID:", parsed?.args.tokenId.toString());
  }
};

task("mint-nft", "Mint a Universal NFT")
  .addParam("contract", "Contract address")
  .addParam("to", "Recipient address")
  .addParam("uri", "Token URI/metadata URL")
  .addOptionalParam("rpc", "RPC URL", "http://localhost:8545")
  .addOptionalParam("privateKey", "Private key")
  .setAction(main);
