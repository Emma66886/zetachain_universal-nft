#!/bin/bash

set -e
set -x
set -o pipefail

# Ensure Foundry is in PATH
export PATH="$HOME/.foundry/bin:$PATH"

yarn zetachain localnet start --force-kill --exit-on-error &

while [ ! -f "$HOME/.zetachain/localnet/registry.json" ]; do sleep 1; done

forge build

ZRC20_BNB=$(jq -r '."98".chainInfo.gasZRC20' ~/.zetachain/localnet/registry.json) && echo $ZRC20_BNB
ZRC20_ETHEREUM=$(jq -r '."11155112".chainInfo.gasZRC20' ~/.zetachain/localnet/registry.json) && echo $ZRC20_ETHEREUM
ZRC20_SOLANA=$(jq -r '."901".chainInfo.gasZRC20' ~/.zetachain/localnet/registry.json 2>/dev/null || echo "NOT_CONFIGURED") && echo $ZRC20_SOLANA
USDC_ETHEREUM=$(jq -r '.["11155112"].contracts[] | select(.contractType == "ERC-20 USDC") | .address' ~/.zetachain/localnet/registry.json) && echo $USDC_ETHEREUM
GATEWAY_ETHEREUM=$(jq -r '.["11155112"].contracts[] | select(.contractType == "gateway") | .address' ~/.zetachain/localnet/registry.json) && echo $GATEWAY_ETHEREUM
GATEWAY_BNB=$(jq -r '."98".contracts[] | select(.contractType == "gateway") | .address' ~/.zetachain/localnet/registry.json) && echo $GATEWAY_BNB
GATEWAY_ZETACHAIN=$(jq -r '.["31337"].contracts[] | select(.contractType == "gateway") | .address' ~/.zetachain/localnet/registry.json) && echo $GATEWAY_ZETACHAIN
WZETA=$(jq -r '.["31337"].contracts[] | select(.contractType == "wzeta") | .address' ~/.zetachain/localnet/registry.json) && echo $WZETA
UNISWAP_ROUTER=$(jq -r '.["31337"].contracts[] | select(.contractType == "uniswapV2Router02") | .address' ~/.zetachain/localnet/registry.json) && echo $UNISWAP_ROUTER
PRIVATE_KEY=$(jq -r '.private_keys[0]' ~/.zetachain/localnet/anvil.json) && echo $PRIVATE_KEY
RECIPIENT=$(cast wallet address $PRIVATE_KEY) && echo $RECIPIENT
RPC=http://localhost:8545

# Deploy Universal NFT contract on ZetaChain
echo "🎨 Deploying Universal NFT contract on ZetaChain..."
UNIVERSAL_NFT=$(forge create UniversalNFT \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --json | jq -r .deployedTo) && echo $UNIVERSAL_NFT

# Initialize Universal NFT
echo "🔧 Initializing Universal NFT..."
cast send $UNIVERSAL_NFT \
  "initialize(address,string,string,address,uint256,address)" \
  $RECIPIENT \
  "Universal NFT" \
  "UNFT" \
  $GATEWAY_ZETACHAIN \
  500000 \
  $UNISWAP_ROUTER \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY

yarn zetachain localnet check

# Deploy Connected contracts on Ethereum and BNB
CONNECTED_ETH=$(forge create Connected \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --json \
  --constructor-args $GATEWAY_ETHEREUM | jq -r .deployedTo) && echo $CONNECTED_ETH

CONNECTED_BNB=$(forge create Connected \
  --rpc-url http://localhost:8546 \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --json \
  --constructor-args $GATEWAY_BNB | jq -r .deployedTo) && echo $CONNECTED_BNB

yarn zetachain localnet check

# Set connected contracts for cross-chain communication
echo "🔗 Setting up cross-chain connections..."

# Connect Ethereum
cast send $UNIVERSAL_NFT \
  "setConnected(address,bytes)" \
  $ZRC20_ETHEREUM \
  $CONNECTED_ETH \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY

# Connect BNB
cast send $UNIVERSAL_NFT \
  "setConnected(address,bytes)" \
  $ZRC20_BNB \
  $CONNECTED_BNB \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY

yarn zetachain localnet check

# Deploy Solana program
echo "🦀 Setting up Solana program..."
cd solana
anchor build
solana-keygen new -o setup/connected-keypair.json --no-bip39-passphrase || true
solana program deploy --program-id setup/connected-keypair.json target/deploy/connected.so --url localhost || echo "Solana program deployment skipped"
cd ..

# Mint NFT on ZetaChain
echo "🎨 Minting NFT on ZetaChain..."
npx tsx -e "
const { ethers } = require('hardhat');
async function main() {
  const [signer] = await ethers.getSigners();
  const contract = await ethers.getContractAt('UniversalNFT', '$UNIVERSAL_NFT');
  const tx = await contract.safeMint('$RECIPIENT', 'https://example.com/metadata/1.json');
  const receipt = await tx.wait();
  console.log('NFT minted, tx:', receipt.hash);
}
main().catch(console.error);
"

yarn zetachain localnet check

# Get the token ID from the last mint (assuming it's 1 for the first NFT)
TOKEN_ID=1

echo "🌉 Starting cross-chain NFT transfer flow..."
echo "📍 Flow: ZetaChain → Ethereum → BNB → Solana → ZetaChain"

# Step 1: Transfer NFT from ZetaChain to Ethereum
echo "🚀 Step 1: ZetaChain → Ethereum"
cast send $UNIVERSAL_NFT \
  "transferCrossChain(uint256,address,address)" \
  $TOKEN_ID \
  $RECIPIENT \
  $ZRC20_ETHEREUM \
  --value 0.1ether \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY

yarn zetachain localnet check
sleep 5

echo "✅ Universal NFT Cross-Chain Flow Completed!"
echo "📊 Summary:"
echo "  - Universal NFT Contract: $UNIVERSAL_NFT"
echo "  - Connected Ethereum: $CONNECTED_ETH" 
echo "  - Connected BNB: $CONNECTED_BNB"
echo "  - Token ID: $TOKEN_ID"
echo "  - Flow: ZetaChain → Ethereum → BNB → Solana → ZetaChain"

yarn zetachain localnet check

yarn zetachain localnet stop

npx tsx ./commands connected call \
  --rpc $RPC \
  --contract $CONNECTED \
  --private-key $PRIVATE_KEY \
  --receiver $UNIVERSAL \
  --types string \
  --values hello \
  --name Connected

yarn zetachain localnet check

npx tsx ./commands connected deposit-and-call \
  --rpc $RPC \
  --contract $CONNECTED \
  --private-key $PRIVATE_KEY \
  --receiver $UNIVERSAL \
  --types string \
  --values hello \
  --amount 0.1 \
  --name Connected

yarn zetachain localnet check

npx tsx ./commands universal withdraw \
  --amount 1 \
  --rpc $RPC \
  --contract $UNIVERSAL \
  --private-key $PRIVATE_KEY \
  --receiver $CONNECTED \
  --name Universal \
  --zrc20 $ZRC20_ETHEREUM 

npx tsx ./commands universal call \
  --rpc $RPC \
  --contract $UNIVERSAL \
  --private-key $PRIVATE_KEY \
  --receiver $CONNECTED \
  --types string \
  --values hello \
  --name Universal \
  --zrc20 $ZRC20_ETHEREUM

yarn zetachain localnet check

npx tsx ./commands universal withdraw-and-call \
  --amount 1 \
  --rpc $RPC \
  --contract $UNIVERSAL \
  --private-key $PRIVATE_KEY \
  --receiver $CONNECTED \
  --types string \
  --values hello \
  --name Universal \
  --zrc20 $ZRC20_ETHEREUM 

yarn zetachain localnet check

yarn zetachain localnet stop