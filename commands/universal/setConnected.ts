import { task } from "hardhat/config";
import type { HardhatRuntimeEnvironment } from "hardhat/types";
import { UniversalNFT } from "../../types";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const [signer] = await hre.ethers.getSigners();
  console.log("🔑 Using account:", signer.address);

  const factory = await hre.ethers.getContractFactory("UniversalNFT");
  const contract = factory.attach(args.contract) as UniversalNFT;

  console.log("🔗 Setting connected chain...");
  console.log("🪙 ZRC-20:", args.zrc20);
  console.log("📍 Contract Address:", args.contractAddress);

  const tx = await contract.setConnected(args.zrc20, args.contractAddress);
  const receipt = await tx.wait();

  console.log("✅ Connected chain set successfully");
  console.log("📄 Transaction hash:", receipt?.hash);
};

task("set-connected", "Set connected contract address for a ZRC-20")
  .addParam("contract", "UniversalNFT contract address")
  .addParam("zrc20", "ZRC-20 token address")
  .addParam("contractAddress", "Connected contract address (hex bytes)")
  .addOptionalParam("rpc", "RPC URL", "http://localhost:8545")
  .addOptionalParam("privateKey", "Private key")
  .setAction(main);
