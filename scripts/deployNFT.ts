import { ethers, upgrades } from "hardhat";
import { HardhatRuntimeEnvironment } from "hardhat/types";

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("🚀 Deploying Universal NFT contracts with account:", deployer.address);
  console.log("💰 Account balance:", (await deployer.provider.getBalance(deployer.address)).toString());

  // Get deployment parameters from environment or use defaults
  const gatewayAddress = process.env.GATEWAY_ADDRESS || "0x9A676e781A523b5d0C0e43731313A708CB607508";
  const uniswapRouter = process.env.UNISWAP_ROUTER || "0x2ca7d64A7EFE2D62A725E2B35Cf7230D6677FfEe";
  const gasLimit = process.env.GAS_LIMIT || "500000";

  console.log("📝 Deployment parameters:");
  console.log("  - Gateway:", gatewayAddress);
  console.log("  - Uniswap Router:", uniswapRouter);  
  console.log("  - Gas Limit:", gasLimit);

  // Deploy Universal NFT (upgradeable)
  console.log("\n🎨 Deploying Universal NFT...");
  const UniversalNFT = await ethers.getContractFactory("UniversalNFT");
  
  const universalNFT = await upgrades.deployProxy(
    UniversalNFT,
    [
      deployer.address, // initialOwner
      "Universal NFT",  // name
      "UNFT",          // symbol
      gatewayAddress,   // gatewayAddress
      gasLimit,        // gas
      uniswapRouter    // uniswapRouterAddress
    ],
    { 
      initializer: "initialize",
      kind: "uups"
    }
  );

  await universalNFT.waitForDeployment();
  const universalNFTAddress = await universalNFT.getAddress();
  
  console.log("✅ Universal NFT deployed to:", universalNFTAddress);

  // Deploy Connected NFT (for connected chains)
  console.log("\n🔗 Deploying Connected NFT...");
  const ConnectedNFT = await ethers.getContractFactory("ConnectedNFT");
  
  const connectedNFT = await ConnectedNFT.deploy(
    gatewayAddress,
    "Connected NFT",
    "CNFT"
  );

  await connectedNFT.waitForDeployment();
  const connectedNFTAddress = await connectedNFT.getAddress();
  
  console.log("✅ Connected NFT deployed to:", connectedNFTAddress);

  // Verification info
  console.log("\n🔍 Verification commands:");
  console.log(`npx hardhat verify --network ${process.env.HARDHAT_NETWORK || 'localhost'} ${universalNFTAddress}`);
  console.log(`npx hardhat verify --network ${process.env.HARDHAT_NETWORK || 'localhost'} ${connectedNFTAddress} "${gatewayAddress}" "Connected NFT" "CNFT"`);

  // Export addresses
  const deploymentInfo = {
    network: process.env.HARDHAT_NETWORK || 'localhost',
    universalNFT: universalNFTAddress,
    connectedNFT: connectedNFTAddress,
    deployer: deployer.address,
    gateway: gatewayAddress,
    uniswapRouter,
    gasLimit,
    timestamp: new Date().toISOString()
  };

  console.log("\n📋 Deployment Summary:");
  console.table(deploymentInfo);

  return deploymentInfo;
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
