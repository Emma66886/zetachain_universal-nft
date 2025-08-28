# Universal NFT Cross-Chain System

This project implements a Universal NFT system that enables seamless cross-chain NFT transfers using ZetaChain's omnichain infrastructure with a burn-and-mint mechanism.

## Features

✅ **Universal NFTs**: NFTs that can exist and be transferred across multiple blockchain networks  
✅ **Burn-and-Mint Mechanism**: Ensures uniqueness - NFTs are burned on source chain and minted on destination  
✅ **Metadata Preservation**: URI and ownership information preserved across chains  
✅ **Multi-Chain Support**: Ethereum, BNB Chain, Solana, and ZetaChain  
✅ **Unique Token IDs**: Globally unique token identifiers across all chains  
✅ **Upgradeable Contracts**: Uses OpenZeppelin's upgradeable proxy pattern  

## Architecture

### EVM Contracts (ZetaChain, Ethereum, BNB)

- **`UniversalNFT.sol`**: Main NFT contract on ZetaChain with cross-chain capabilities
- **`UniversalNFTCore.sol`**: Core logic for cross-chain transfers and burn-mint mechanism  
- **`ConnectedNFT.sol`**: Connected chain contracts for Ethereum and BNB
- **`UniversalNFTEvents.sol`**: Event definitions for cross-chain operations

### Solana Program

- **`connected`**: Rust/Anchor program handling NFT operations on Solana
- Supports minting, burning, and cross-chain transfers
- Compatible with SPL Token and Metaplex metadata standards

## Cross-Chain Flow

```
ZetaChain ←→ Ethereum ←→ BNB Chain ←→ Solana ←→ ZetaChain
```

1. **Mint**: NFT can be minted on any connected chain
2. **Transfer**: Burn NFT on source chain, emit cross-chain message  
3. **Receive**: Mint NFT on destination chain with same token ID and metadata
4. **Unique IDs**: Each NFT has a globally unique identifier across all chains

## Setup & Installation

### Prerequisites

- Node.js 18+
- Rust & Anchor (for Solana)
- Foundry (for Ethereum contracts)

### Install Dependencies

```bash
npm install
# or
yarn install
```

### Build Contracts

```bash
# Build EVM contracts
forge build

# Build Solana program
cd solana
anchor build
cd ..
```

## Usage

### Local Development

Start the complete cross-chain NFT system:

```bash
npm run localnet:nft
```

This will:
1. Start ZetaChain localnet
2. Deploy Universal NFT contracts
3. Deploy connected contracts on Ethereum and BNB
4. Set up Solana program
5. Demonstrate cross-chain NFT transfer flow

### NFT CLI Commands

#### Mint NFT
```bash
npm run nft:mint -- \
  --contract 0x... \
  --to 0x742d35Cc6361C2CD6D7D76bafF0726cdC3 \
  --uri https://example.com/metadata/1.json
```

#### Transfer Cross-Chain
```bash
npm run nft:transfer -- \
  --contract 0x... \
  --token-id 1 \
  --receiver 0x742d35Cc6361C2CD6D7D76bafF0726cdC3 \
  --destination 0x... \
  --amount 0.1
```

#### Set Connected Chain
```bash
npm run nft:set-connected -- \
  --contract 0x... \
  --zrc20 0x... \
  --address 0x...
```

#### Get NFT Info
```bash
npm run nft:info -- \
  --contract 0x... \
  --token-id 1
```

### Manual Contract Interaction

```javascript
// Mint NFT
const tx = await universalNFT.safeMint(recipient, "https://metadata.json");

// Transfer cross-chain
const tx = await universalNFT.transferCrossChain(
  tokenId,
  receiverBytes32,
  destinationZRC20,
  { value: ethers.parseEther("0.1") }
);
```

## Solana Integration

### Program Operations

```typescript
// Initialize NFT state
await program.methods.initialize().rpc();

// Mint NFT
await program.methods.mintNft(
  tokenId, name, symbol, uri, recipient
).rpc();

// Burn for cross-chain transfer  
await program.methods.burnNft(
  tokenId, "ethereum", "0x..."
).rpc();
```

### Address Conversion

Solana addresses (base58) are converted to bytes32 for EVM compatibility:

```javascript
const bs58 = require("bs58");
const pubkeyBytes = bs58.decode(solanaAddress);
const bytes32 = "0x" + Buffer.from(pubkeyBytes).toString("hex").padStart(64, "0");
```

## Testing

### Run Local Demo
```bash
npm run nft:demo
```

### Test Cross-Chain Flow
```bash
# This will test the complete NFT journey across all chains
npm run localnet:nft
```

## Key Concepts

### Burn-and-Mint Mechanism
- **Burning**: NFT is permanently destroyed on the source chain
- **Minting**: New NFT created on destination with same token ID and metadata
- **Uniqueness**: Ensures only one instance exists across all chains

### Global Token IDs
```solidity
// Generate unique token ID
uint256 hash = uint256(
    keccak256(abi.encodePacked(address(this), block.number, _nextTokenId++))
);
uint256 tokenId = hash & 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
```

### Cross-Chain Message Format
```solidity
bytes memory message = abi.encode(
    receiver,      // bytes32: destination address  
    tokenId,       // uint256: unique token identifier
    uri,           // string: metadata URI
    gasAmount,     // uint256: gas for execution
    originalSender // address: original owner
);
```

## Security Considerations

- ✅ Only authorized contracts can mint/burn NFTs
- ✅ Ownership verification before burning
- ✅ Reentrancy protection on all functions
- ✅ Gas limit controls for cross-chain calls
- ✅ Proper access controls with OpenZeppelin

## Troubleshooting

### Common Issues

1. **Transaction fails**: Check gas limits and token allowances
2. **Cross-chain transfer stuck**: Verify connected contract addresses  
3. **Solana program fails**: Ensure all accounts are properly initialized
4. **Metadata not loading**: Verify URI is accessible and properly formatted

### Debug Commands

```bash
# Check contract state
npx hardhat console --network localhost

# View Solana program logs
solana logs --url localhost

# Monitor cross-chain events
npx tsx scripts/eventListener.ts
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Links

- [ZetaChain Docs](https://docs.zetachain.com)
- [OpenZeppelin Contracts](https://docs.openzeppelin.com/contracts)
- [Anchor Framework](https://anchor-lang.com)
- [Metaplex Metadata](https://docs.metaplex.com)
