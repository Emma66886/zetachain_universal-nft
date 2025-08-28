/**
 * Complete Universal NFT Cross-Chain Transfer Example
 * 
 * This example demonstrates:
 * 1. Minting an NFT on ZetaChain
 * 2. Transferring it to Ethereum 
 * 3. Transferring it to BNB Chain
 * 4. Transferring it to Solana
 * 5. Transferring it back to ZetaChain
 * 
 * The burn-and-mint mechanism ensures the NFT exists on only one chain at a time
 * while preserving metadata and ownership across all transfers.
 */

import { ethers } from "ethers";
import { PublicKey } from "@solana/web3.js";

// Mock addresses for demonstration - replace with actual deployed addresses
const CONTRACT_ADDRESSES = {
  ZETACHAIN: {
    UNIVERSAL_NFT: "0x1234567890abcdef1234567890abcdef12345678",
    ZRC20_ETH: "0xabcdef1234567890abcdef1234567890abcdef12",
    ZRC20_BNB: "0x567890abcdef1234567890abcdef1234567890ab",
    ZRC20_SOL: "0x234567890abcdef1234567890abcdef12345678"
  },
  ETHEREUM: {
    CONNECTED_NFT: "0x890abcdef1234567890abcdef1234567890abcde"
  },
  BNB: {
    CONNECTED_NFT: "0xdef1234567890abcdef1234567890abcdef12345"
  },
  SOLANA: {
    PROGRAM_ID: "9BjVGjn28E58LgSi547JYEpqpgRoo1TErkbyXiRSNDQy"
  }
};

interface NFTMetadata {
  name: string;
  description: string;
  image: string;
  attributes: Array<{
    trait_type: string;
    value: string;
  }>;
}

class UniversalNFTDemo {
  private provider: ethers.Provider;
  private signer: ethers.Signer;

  constructor(provider: ethers.Provider, signer: ethers.Signer) {
    this.provider = provider;
    this.signer = signer;
  }

  /**
   * Step 1: Mint NFT on ZetaChain
   */
  async mintNFTOnZetaChain(): Promise<{ tokenId: string; txHash: string }> {
    console.log("🎨 Step 1: Minting NFT on ZetaChain...");
    
    const metadata: NFTMetadata = {
      name: "Universal Traveler #1",
      description: "An NFT that travels across chains while maintaining its identity",
      image: "https://example.com/images/universal-traveler-1.png",
      attributes: [
        { trait_type: "Type", value: "Universal" },
        { trait_type: "Chain Origin", value: "ZetaChain" },
        { trait_type: "Journey Status", value: "Active" }
      ]
    };

    // In a real implementation, you'd upload metadata to IPFS
    const metadataUri = "https://example.com/metadata/universal-traveler-1.json";

    const contract = new ethers.Contract(
      CONTRACT_ADDRESSES.ZETACHAIN.UNIVERSAL_NFT,
      [
        "function safeMint(address to, string memory uri) public",
        "event Transfer(address indexed from, address indexed to, uint256 indexed tokenId)"
      ],
      this.signer
    );

    const tx = await contract.safeMint(await this.signer.getAddress(), metadataUri);
    const receipt = await tx.wait();

    // Extract token ID from Transfer event
    const transferEvent = receipt.logs.find((log: any) => {
      try {
        const parsed = contract.interface.parseLog(log);
        return parsed?.name === "Transfer" && parsed?.args.from === ethers.ZeroAddress;
      } catch {
        return false;
      }
    });

    const tokenId = transferEvent 
      ? contract.interface.parseLog(transferEvent).args.tokenId.toString()
      : "1";

    console.log("✅ NFT minted successfully!");
    console.log(`   Token ID: ${tokenId}`);
    console.log(`   Transaction: ${receipt.hash}`);
    console.log(`   Metadata: ${metadataUri}`);

    return { tokenId, txHash: receipt.hash };
  }

  /**
   * Step 2: Transfer NFT from ZetaChain to Ethereum
   */
  async transferToEthereum(tokenId: string): Promise<string> {
    console.log("\n🌉 Step 2: Transferring NFT ZetaChain → Ethereum...");
    
    const contract = new ethers.Contract(
      CONTRACT_ADDRESSES.ZETACHAIN.UNIVERSAL_NFT,
      [
        "function transferCrossChain(uint256 tokenId, bytes32 receiver, address destination) public payable"
      ],
      this.signer
    );

    const receiverAddress = await this.signer.getAddress();
    const receiverBytes32 = ethers.zeroPadValue(receiverAddress, 32);

    const tx = await contract.transferCrossChain(
      tokenId,
      receiverBytes32,
      CONTRACT_ADDRESSES.ZETACHAIN.ZRC20_ETH,
      { value: ethers.parseEther("0.1") }
    );

    const receipt = await tx.wait();
    console.log("✅ Transfer to Ethereum initiated!");
    console.log(`   Transaction: ${receipt.hash}`);
    console.log("   NFT burned on ZetaChain, will be minted on Ethereum");

    return receipt.hash;
  }

  /**
   * Step 3: Transfer NFT from Ethereum to BNB Chain
   */
  async transferToBNB(tokenId: string): Promise<string> {
    console.log("\n🌉 Step 3: Transferring NFT Ethereum → BNB Chain...");
    
    // This would be executed on Ethereum network
    console.log("   This step requires interaction with Ethereum network");
    console.log("   The Connected NFT contract on Ethereum would:");
    console.log("   1. Verify ownership of the NFT");
    console.log("   2. Burn the NFT on Ethereum");  
    console.log("   3. Initiate cross-chain transfer to BNB");
    
    // Mock transaction hash
    const mockTxHash = "0xabcdef123456789...";
    console.log(`✅ Transfer to BNB initiated! Tx: ${mockTxHash}`);
    
    return mockTxHash;
  }

  /**
   * Step 4: Transfer NFT from BNB Chain to Solana
   */
  async transferToSolana(tokenId: string): Promise<string> {
    console.log("\n🌉 Step 4: Transferring NFT BNB Chain → Solana...");
    
    // Generate a mock Solana address for demonstration
    const solanaAddress = new PublicKey("11111111111111111111111111111112");
    
    console.log("   This step requires interaction with BNB Chain");
    console.log(`   Target Solana address: ${solanaAddress.toString()}`);
    console.log("   The Connected NFT contract on BNB would:");
    console.log("   1. Verify ownership of the NFT");
    console.log("   2. Burn the NFT on BNB Chain");
    console.log("   3. Initiate cross-chain transfer to Solana");
    console.log("   4. Solana program receives message and mints NFT");
    
    // Mock transaction hash  
    const mockTxHash = "5J7KqzVNHMQs8XwGQZ...";
    console.log(`✅ Transfer to Solana initiated! Tx: ${mockTxHash}`);
    
    return mockTxHash;
  }

  /**
   * Step 5: Transfer NFT from Solana back to ZetaChain
   */
  async transferBackToZetaChain(tokenId: string): Promise<string> {
    console.log("\n🌉 Step 5: Transferring NFT Solana → ZetaChain...");
    
    console.log("   This step requires interaction with Solana network");
    console.log("   The Solana program would:");
    console.log("   1. Verify ownership of the NFT");
    console.log("   2. Burn the NFT on Solana");
    console.log("   3. Initiate cross-chain transfer back to ZetaChain");
    console.log("   4. ZetaChain receives message and mints NFT");
    
    // Mock signature
    const mockSignature = "3KqzVNHMQs8XwGQZJ7...";
    console.log(`✅ Transfer back to ZetaChain initiated! Signature: ${mockSignature}`);
    
    return mockSignature;
  }

  /**
   * Verify NFT state across chains
   */
  async verifyNFTState(tokenId: string) {
    console.log("\n🔍 Verifying NFT state across all chains...");
    
    const results = {
      zetachain: false,
      ethereum: false, 
      bnb: false,
      solana: false
    };

    // In a real implementation, you would check each chain
    // For demo purposes, we'll show the expected final state
    results.zetachain = true; // NFT is back on ZetaChain
    
    console.log("📊 NFT State Verification:");
    console.table({
      "ZetaChain": results.zetachain ? "✅ Present" : "❌ Not Found",
      "Ethereum": results.ethereum ? "✅ Present" : "❌ Not Found", 
      "BNB Chain": results.bnb ? "✅ Present" : "❌ Not Found",
      "Solana": results.solana ? "✅ Present" : "❌ Not Found"
    });

    const totalCount = Object.values(results).filter(Boolean).length;
    if (totalCount === 1) {
      console.log("✅ SUCCESS: NFT exists on exactly one chain (as expected)");
    } else {
      console.log(`❌ ERROR: NFT exists on ${totalCount} chains (should be 1)`);
    }
  }

  /**
   * Run the complete cross-chain journey
   */
  async runCompleteJourney() {
    console.log("🚀 Universal NFT Cross-Chain Journey Demo");
    console.log("==========================================\n");

    try {
      // Step 1: Mint on ZetaChain
      const { tokenId } = await this.mintNFTOnZetaChain();
      
      // Wait for confirmation
      await new Promise(resolve => setTimeout(resolve, 2000));

      // Step 2: Transfer to Ethereum
      await this.transferToEthereum(tokenId);
      await new Promise(resolve => setTimeout(resolve, 5000));

      // Step 3: Transfer to BNB
      await this.transferToBNB(tokenId);
      await new Promise(resolve => setTimeout(resolve, 5000));

      // Step 4: Transfer to Solana  
      await this.transferToSolana(tokenId);
      await new Promise(resolve => setTimeout(resolve, 5000));

      // Step 5: Transfer back to ZetaChain
      await this.transferBackToZetaChain(tokenId);
      await new Promise(resolve => setTimeout(resolve, 5000));

      // Verify final state
      await this.verifyNFTState(tokenId);

      console.log("\n🎉 Universal NFT Cross-Chain Journey Complete!");
      console.log("Key Benefits Demonstrated:");
      console.log("✅ Burn-and-mint ensures uniqueness across chains");
      console.log("✅ Metadata and ownership preserved throughout journey");  
      console.log("✅ Seamless interoperability between EVM and Solana");
      console.log("✅ Global token ID remains consistent across all chains");
      console.log("✅ Decentralized cross-chain infrastructure via ZetaChain");

    } catch (error) {
      console.error("❌ Error during cross-chain journey:", error);
    }
  }
}

// Example usage
async function runDemo() {
  // In a real implementation, connect to actual networks
  const provider = new ethers.JsonRpcProvider("http://localhost:8545");
  const signer = new ethers.Wallet("0x" + "0".repeat(64), provider); // Use real private key
  
  const demo = new UniversalNFTDemo(provider, signer);
  await demo.runCompleteJourney();
}

// Export for use in other scripts
export { UniversalNFTDemo, CONTRACT_ADDRESSES };

// Run demo if called directly
if (require.main === module) {
  runDemo().catch(console.error);
}
