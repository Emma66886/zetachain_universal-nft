# ZetaChain Gateway Integration for Universal NFT

This document explains how our Universal NFT solution integrates with the official ZetaChain Gateway protocol for production-ready cross-chain transfers.

## Gateway Integration Overview

Our Solana program is designed to integrate with the official ZetaChain Gateway program using Cross-Program Invocation (CPI) patterns documented in the official ZetaChain protocol contracts.

## Official Gateway Integration Pattern

Based on analysis of the official ZetaChain protocol contracts, the proper Gateway integration follows this pattern:

### 1. Gateway Program Structure

The official ZetaChain Gateway program (ID: `ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis`) provides these key functions:

```rust
// For SPL token deposits with contract calls
pub fn deposit_spl_token_and_call(
    ctx: Context<DepositSplToken>,
    amount: u64,
    receiver: [u8; 20],
    message: Vec<u8>,
    revert_options: Option<RevertOptions>,
) -> Result<()>
```

### 2. Required Account Context

The `DepositSplToken` context from the official Gateway requires:

```rust
#[derive(Accounts)]
pub struct DepositSplToken<'info> {
    /// The signer making the deposit
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Gateway PDA
    #[account(mut, seeds = [b"meta"], bump)]
    pub pda: Account<'info, Pda>,

    /// Whitelist entry for the SPL token
    #[account(seeds = [b"whitelist", mint_account.key().as_ref()], bump)]
    pub whitelist_entry: Account<'info, WhitelistEntry>,

    /// The mint account of the SPL token
    pub mint_account: Account<'info, Mint>,

    /// Token program
    pub token_program: Program<'info, Token>,

    /// Source token account (user's)
    #[account(mut, constraint = from.mint == mint_account.key())]
    pub from: Account<'info, TokenAccount>,

    /// Destination token account (Gateway's)
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,

    /// System program
    pub system_program: Program<'info, System>,
}
```

### 3. Caller Verification Pattern

The official connected program example shows proper caller verification:

```rust
let current_ix = anchor_lang::solana_program::sysvar::instructions::get_instruction_relative(0, &ctx.accounts.instruction_sysvar)?;
require!(current_ix.program_id == gateway::ID, ErrorCode::InvalidCaller);
```

## Our Implementation

### Current Status

Our `transfer_cross_chain` function demonstrates the complete Gateway integration pattern:

1. **Burn-and-Mint Mechanism**: NFTs are burned on the source chain before cross-chain transfer
2. **Message Serialization**: Cross-chain messages are properly serialized for ZetaChain
3. **Event Emission**: Comprehensive event logging for cross-chain tracking
4. **Error Handling**: Proper error codes and revert mechanisms

### Production Integration

To enable full Gateway integration, add the Gateway dependency to `Cargo.toml`:

```toml
[dependencies]
gateway = { git = "https://github.com/zeta-chain/protocol-contracts-solana.git", features = ["no-entrypoint", "cpi"] }
```

Then update the `transfer_cross_chain` function to make the actual CPI call:

```rust
// Create CPI context for Gateway deposit call
let gateway_cpi_accounts = gateway::cpi::accounts::DepositSplToken {
    signer: ctx.accounts.signer.to_account_info(),
    pda: ctx.accounts.gateway_pda.to_account_info(),
    whitelist_entry: ctx.accounts.whitelist_entry.to_account_info(),
    mint_account: ctx.accounts.mint.to_account_info(),
    token_program: ctx.accounts.token_program.to_account_info(),
    from: ctx.accounts.token_account.to_account_info(),
    to: ctx.accounts.gateway_token_account.to_account_info(),
    system_program: ctx.accounts.system_program.to_account_info(),
};

let gateway_cpi_ctx = CpiContext::new(
    ctx.accounts.gateway_program.to_account_info(),
    gateway_cpi_accounts,
);

// Create revert options for cross-chain call
let revert_options = Some(gateway::RevertOptions {
    revert_address: ctx.accounts.gateway_pda.key().as_ref().try_into().unwrap(),
    call_on_revert: true,
    abort_address: ctx.accounts.gateway_pda.key().as_ref().try_into().unwrap(),
    revert_message: b"NFT transfer failed".to_vec(),
    on_revert_gas_limit: 100000,
});

// Call Gateway deposit_spl_token_and_call for cross-chain transfer
gateway::cpi::deposit_spl_token_and_call(
    gateway_cpi_ctx,
    1, // amount (1 NFT)
    recipient_address,
    serialized_message,
    revert_options,
)?;
```

## Account Setup for Production

### Required Accounts for Gateway CPI

1. **Gateway Program**: `ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis`
2. **Gateway PDA**: Seeds `[b"meta"]` with Gateway program
3. **Whitelist Entry**: Seeds `[b"whitelist", mint.key()]` with Gateway program
4. **Gateway Token Account**: Associated token account for Gateway PDA
5. **Instructions Sysvar**: For caller verification

### Account Constraints

```rust
#[derive(Accounts)]
#[instruction(token_id: u64)]
pub struct TransferCrossChain<'info> {
    // ... existing accounts ...
    
    /// ZetaChain Gateway program
    /// CHECK: Must be official Gateway program ID
    #[account(address = gateway::ID)]
    pub gateway_program: AccountInfo<'info>,
    
    /// Gateway PDA
    #[account(
        mut,
        seeds = [b"meta"],
        bump,
        seeds::program = gateway_program.key()
    )]
    pub gateway_pda: Account<'info, gateway::Pda>,
    
    /// Whitelist entry for NFT mint
    #[account(
        seeds = [b"whitelist", mint.key().as_ref()],
        bump,
        seeds::program = gateway_program.key()
    )]
    pub whitelist_entry: Account<'info, gateway::WhitelistEntry>,
    
    /// Gateway token account
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = gateway_pda
    )]
    pub gateway_token_account: Account<'info, TokenAccount>,
}
```

## Security Considerations

1. **Caller Verification**: Always verify calls originate from authorized programs
2. **Account Ownership**: Validate all accounts are owned by correct programs
3. **Token Validation**: Ensure tokens are properly whitelisted in Gateway
4. **Revert Handling**: Implement comprehensive revert mechanisms
5. **Message Validation**: Validate cross-chain message format and size

## Testing with Localnet

For testing, the Gateway program provides a development version:

```rust
#[cfg(feature = "dev")]
declare_id!("94U5AHQMKkV5txNJ17QPXWoh474PheGou6cNP2FEuL1d");
```

This allows testing cross-chain functionality without mainnet deployment.

## Cross-Chain Message Format

Our Universal NFT uses this message structure for cross-chain transfers:

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CrossChainMessage {
    pub message_type: MessageType,
    pub token_id: u64,
    pub recipient_address: [u8; 20], // EVM address
    pub metadata_uri: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum MessageType {
    Mint,    // Create NFT on destination chain
    Burn,    // Remove NFT on destination chain
    Transfer, // Transfer existing NFT
}
```

## Integration Status

- ✅ **Account Structure**: Proper Gateway account constraints defined
- ✅ **Message Format**: Cross-chain message serialization implemented
- ✅ **Burn Mechanism**: NFT burning on source chain implemented
- ✅ **Event Emission**: Comprehensive event logging implemented
- ✅ **Error Handling**: Proper error codes and validation
- 🔄 **CPI Integration**: Ready for Gateway dependency addition
- 🔄 **Production Testing**: Requires Gateway program deployment

This integration pattern ensures compatibility with the official ZetaChain protocol while maintaining the security and functionality required for production Universal NFT cross-chain transfers.
