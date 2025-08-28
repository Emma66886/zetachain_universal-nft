# ZetaChain Solana Integration Updates

This document outlines the updates made to the Solana Universal NFT program to align with the official ZetaChain documentation.

## Key Updates Made

### 1. Updated RevertOptions Structure
**Before:**
```rust
pub struct RevertOptions {
    pub revert_address: Pubkey,
    pub call_on_revert: bool,
    pub abort_address: Pubkey,
    pub revert_message: Vec<u8>,
}
```

**After (Official ZetaChain Structure):**
```rust
pub struct RevertOptions {
    pub revert_address: Pubkey,
    pub abort_address: [u8; 20],      // 20-byte Ethereum-style address
    pub call_on_revert: bool,
    pub revert_message: Vec<u8>,
    pub on_revert_gas_limit: u64,     // New field for revert gas limit
}
```

### 2. Updated on_call Function Signature
**Official ZetaChain Signature:**
```rust
pub fn on_call(
    ctx: Context<OnCall>,
    amount: u64,        // Amount of tokens being withdrawn/deposited
    sender: [u8; 20],   // Address of the universal app on ZetaChain
    data: Vec<u8>,      // Additional data passed from the universal app
) -> Result<()>
```

### 3. Added on_revert Function
**New Function (Per Official Documentation):**
```rust
pub fn on_revert(
    ctx: Context<OnRevert>,
    amount: u64,        // Asset quantity originally deposited (lamports or SPL)
    sender: Pubkey,     // The account that triggered the deposit/call from Solana
    data: Vec<u8>,      // Arbitrary bytes supplied via revert_message
) -> Result<()>
```

### 4. Updated Gateway Call Structure
- Removed `gas_limit` from `GatewayCallInstruction`
- Updated revert options to use proper abort address format
- Added proper gas limit handling through `on_revert_gas_limit`

### 5. Added OnRevert Account Context
```rust
#[derive(Accounts)]
pub struct OnRevert<'info> {
    #[account(mut, seeds = [b"connected"], bump)]
    pub pda: Account<'info, Pda>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
```

### 6. Added CrossChainTransferReverted Event
```rust
#[event]
pub struct CrossChainTransferReverted {
    pub token_id: u64,
    pub original_sender: Pubkey,
    pub reverted_amount: u64,
}
```

## ZetaChain Gateway Integration Patterns

### Supported Gateway Functions:
1. **`deposit`** - Deposit SOL to EOA or universal contract
2. **`deposit_spl_token`** - Deposit SPL tokens
3. **`deposit_and_call`** - Deposit SOL and call universal app
4. **`deposit_spl_token_and_call`** - Deposit SPL tokens and call universal app
5. **`call`** - Call universal app without asset movement

### Key Features Implemented:
- **Burn-and-mint mechanism** for NFT cross-chain transfers
- **Comprehensive error handling** with revert scenarios
- **Event emission** for transfer tracking
- **Official account structures** matching ZetaChain specifications
- **Gas limit handling** for revert transactions

### Error Handling:
- **Revert scenarios** handled via `on_revert` function
- **Asset protection** through abort address fallback
- **Custom revert messages** for debugging and user feedback

## Fees and Limitations
- **Deposit fee**: 2,000,000 lamports (0.002 SOL) for all deposits
- **SPL token support**: Only whitelisted SPL tokens can be deposited
- **Single token deposits**: Only one SPL token can be deposited at a time

## Next Steps
1. Test with actual ZetaChain Gateway program
2. Implement proper account derivation for gateway PDAs
3. Add comprehensive error codes matching ZetaChain standards
4. Test revert scenarios in various failure conditions
