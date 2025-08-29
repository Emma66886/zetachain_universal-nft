use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions;
use std::mem::size_of;
use anchor_spl::{
    token::{self, Mint, Token, TokenAccount, MintTo, mint_to, Burn, burn},
    associated_token::AssociatedToken,
    metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata},
};
use mpl_token_metadata::types::DataV2;
use gateway::{self, RevertOptions};

declare_id!("9BjVGjn28E58LgSi547JYEpqpgRoo1TErkbyXiRSNDQy");

#[program]
pub mod connected {
    use super::*;

    /// Initialize the Universal NFT program
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let universal_nft_state = &mut ctx.accounts.universal_nft_state;
        universal_nft_state.authority = ctx.accounts.signer.key();
        universal_nft_state.total_supply = 0;
        universal_nft_state.next_token_id = 1;
        Ok(())
    }

    /// Mint a new Universal NFT
    pub fn mint_nft(
        ctx: Context<MintNFT>,
        token_id: u64,
        name: String,
        symbol: String,
        uri: String,
        to: Pubkey,
    ) -> Result<()> {
        let universal_nft_state = &mut ctx.accounts.universal_nft_state;
        
        // Ensure token ID is unique
        require!(token_id >= universal_nft_state.next_token_id, UniversalNFTError::TokenIdTaken);
        
        // Create mint account
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        mint_to(cpi_ctx, 1)?;

        // Create metadata
        let data_v2 = DataV2 {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let cpi_accounts = CreateMetadataAccountsV3 {
            metadata: ctx.accounts.metadata.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            mint_authority: ctx.accounts.signer.to_account_info(),
            update_authority: ctx.accounts.signer.to_account_info(),
            payer: ctx.accounts.signer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        let cpi_program = ctx.accounts.metadata_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        create_metadata_accounts_v3(cpi_ctx, data_v2, true, true, None)?;

        // Store NFT information
        let nft_info = &mut ctx.accounts.nft_info;
        nft_info.token_id = token_id;
        nft_info.name = name;
        nft_info.symbol = symbol;
        nft_info.uri = uri;
        nft_info.owner = to;
        nft_info.is_burned = false;
        nft_info.mint = ctx.accounts.mint.key();

        universal_nft_state.total_supply += 1;
        if token_id >= universal_nft_state.next_token_id {
            universal_nft_state.next_token_id = token_id + 1;
        }

        emit!(NFTMinted {
            token_id,
            owner: to,
            uri: nft_info.uri.clone(),
            mint: ctx.accounts.mint.key(),
        });

        Ok(())
    }

    /// Burn NFT for cross-chain transfer
    pub fn burn_nft(
        ctx: Context<BurnNFT>,
        token_id: u64,
        destination_chain: String,
        destination_receiver: String,
    ) -> Result<()> {
        let nft_info = &mut ctx.accounts.nft_info;
        let universal_nft_state = &mut ctx.accounts.universal_nft_state;

        // Verify ownership
        require!(nft_info.owner == ctx.accounts.signer.key(), UniversalNFTError::NotOwner);
        require!(!nft_info.is_burned, UniversalNFTError::AlreadyBurned);

        // Burn the token
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        burn(cpi_ctx, 1)?;

        // Mark as burned
        nft_info.is_burned = true;
        universal_nft_state.total_supply -= 1;

        emit!(NFTBurned {
            token_id,
            owner: nft_info.owner,
            destination_chain,
            destination_receiver,
            uri: nft_info.uri.clone(),
        });

        Ok(())
    }

    /// Handle incoming cross-chain calls from ZetaChain
    /// Official signature from ZetaChain documentation
    pub fn on_call(
        ctx: Context<OnCall>,
        amount: u64,
        sender: [u8; 20],
        data: Vec<u8>,
    ) -> Result<()> {
        // Use amount parameter to track the deposited amount
        msg!("Received cross-chain call with amount: {}", amount);
        
        // Decode the NFT transfer data
        let transfer_data = CrossChainNFTTransfer::deserialize(&mut &data[..])
            .map_err(|_| ErrorCode::DecodingError)?;

        // Mint the NFT on Solana
        let mint_accounts = MintTo {
            mint: ctx.accounts.mint_account.to_account_info(),
            to: ctx.accounts.pda_ata.to_account_info(),
            authority: ctx.accounts.pda.to_account_info(),
        };

        let seeds = &[b"connected".as_ref(), &[ctx.bumps.pda]];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            mint_accounts,
            signer_seeds,
        );

        mint_to(cpi_ctx, 1)?;

        emit!(CrossChainTransferReceived {
            token_id: transfer_data.token_id,
            sender,
            receiver: ctx.accounts.pda.key(),
            name: transfer_data.name,
            symbol: transfer_data.symbol,
            uri: transfer_data.uri,
        });

        Ok(())
    }

    /// Handle transaction reverts from ZetaChain
    /// Official signature from ZetaChain documentation
    pub fn on_revert(
        ctx: Context<OnRevert>,
        amount: u64,        // Asset quantity originally deposited (lamports or SPL)
        sender: Pubkey,     // The account that triggered the deposit/call from Solana
        data: Vec<u8>,      // Arbitrary bytes supplied via revert_message
    ) -> Result<()> {
        // Handle the revert scenario
        // This could involve refunding tokens, updating state, or emitting events
        
        msg!("Cross-chain transaction reverted for PDA: {}", ctx.accounts.pda.key());
        msg!("Original sender: {}", sender);
        msg!("Reverted amount: {}", amount);
        
        // Use the amount parameter to avoid warnings
        let _reverted_amount = amount;
        
        // Attempt to decode the original transfer data if possible
        if let Ok(transfer_data) = CrossChainNFTTransfer::deserialize(&mut &data[..]) {
            msg!("Reverted NFT transfer for token_id: {}", transfer_data.token_id);
            
            // You could implement logic here to:
            // - Restore the burned NFT
            // - Refund any associated tokens
            // - Update application state
            
            emit!(CrossChainTransferReverted {
                token_id: transfer_data.token_id,
                original_sender: sender,
                reverted_amount: _reverted_amount,
            });
        }

        Ok(())
    }
}

// Helper function to decode NFT transfer data
fn decode_nft_transfer(data: &[u8]) -> Result<CrossChainNFTTransfer> {
    CrossChainNFTTransfer::deserialize(&mut &data[..]).map_err(|_| ErrorCode::DecodingError.into())
}

    /// Transfer NFT cross-chain using ZetaChain Gateway
    pub fn transfer_cross_chain(
        ctx: Context<TransferCrossChain>,
        token_id: u64,
        recipient_address: [u8; 20], // Ethereum address on destination chain
        destination_chain_id: u64,
        metadata_uri: String,
    ) -> Result<()> {
        msg!("Starting cross-chain NFT transfer");
        
        // Verify caller authentication (in production, this would verify Gateway program)
        let current_ix = instructions::get_instruction_relative(0, &ctx.accounts.instruction_sysvar)?;
        msg!("Current instruction program ID: {}", current_ix.program_id);
        
        let nft_info = &mut ctx.accounts.nft_info;
        
        // Verify NFT exists and is owned by correct owner
        if nft_info.owner != *ctx.accounts.signer.key {
            return Err(ErrorCode::NotOwner.into());
        }
        
        // Ensure NFT is not already burned
        require!(!nft_info.is_burned, UniversalNFTError::AlreadyBurned);
        
        // Prepare cross-chain message for ZetaChain
        let message_data = CrossChainMessage {
            message_type: MessageType::Mint,
            token_id,
            recipient_address,
            metadata_uri: metadata_uri.clone(),
        };
        
        let serialized_message = message_data.try_to_vec()
            .map_err(|_| ErrorCode::SerializationError)?;
        
        msg!("Serialized cross-chain message: {} bytes", serialized_message.len());
        
        // Burn the NFT on source chain first
        let token_account = &ctx.accounts.token_account;
        let mint_account = &ctx.accounts.mint;
        
        // Burn token using token program
        let cpi_accounts = token::Burn {
            mint: mint_account.to_account_info(),
            from: token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::burn(cpi_ctx, 1)?;
        msg!("NFT burned successfully on source chain");
        
        // Update NFT state to indicate cross-chain transfer
        nft_info.is_burned = true;
        nft_info.cross_chain_data = Some(CrossChainData {
            destination_chain_id,
            recipient_address,
            transfer_timestamp: Clock::get()?.unix_timestamp,
        });
        
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
        let revert_options = Some(RevertOptions {
            revert_address: ctx.accounts.signer.key(),
            call_on_revert: true,
            abort_address: recipient_address,
            revert_message: b"NFT transfer failed".to_vec(),
            on_revert_gas_limit: 100000,
        });
        
        // Call Gateway deposit_spl_token_and_call for cross-chain transfer
        gateway::cpi::deposit_spl_token_and_call(
            gateway_cpi_ctx,
            1, // amount (1 NFT)
            recipient_address,
            serialized_message.clone(),
            revert_options,
        )?;
        
        msg!("Gateway CPI call executed successfully");
        msg!("Amount: 1 NFT token");
        msg!("Recipient: {:?}", recipient_address);
        msg!("Message size: {} bytes", serialized_message.len());
        
        // Emit cross-chain transfer event
        emit!(CrossChainTransferEvent {
            token_id,
            from_chain: "Solana".to_string(),
            to_chain: format!("Chain-{}", destination_chain_id),
            sender: *ctx.accounts.signer.key,
            receiver: recipient_address,
        });
        
        msg!("NFT transferred cross-chain successfully via Gateway pattern");
        msg!("Token ID: {}, Destination Chain: {}", token_id, destination_chain_id);
        msg!("Recipient Address: {:?}", recipient_address);
        
        Ok(())
    }

// Cross-chain message types and data structures
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum MessageType {
    Mint,
    Burn,
    Transfer,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CrossChainMessage {
    pub message_type: MessageType,
    pub token_id: u64,
    pub recipient_address: [u8; 20],
    pub metadata_uri: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CrossChainData {
    pub destination_chain_id: u64,
    pub recipient_address: [u8; 20],
    pub transfer_timestamp: i64,
}

// ZetaChain Gateway integration structs
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GatewayCallInstruction {
    pub receiver: [u8; 20],
    pub message: Vec<u8>,
    pub revert_options: Option<RevertOptions>,
}

// Account contexts

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + size_of::<UniversalNFTState>(),
        seeds = [b"universal_nft_state"],
        bump
    )]
    pub universal_nft_state: Account<'info, UniversalNFTState>,

    #[account(init, payer = signer, space = size_of::<Pda>() + 32, seeds = [b"connected"], bump)]
    pub pda: Account<'info, Pda>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(token_id: u64)]
pub struct MintNFT<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"universal_nft_state"],
        bump
    )]
    pub universal_nft_state: Account<'info, UniversalNFTState>,

    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = signer,
        seeds = [b"nft_mint", token_id.to_le_bytes().as_ref()],
        bump
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        space = 8 + size_of::<NFTInfo>(),
        seeds = [b"nft_info", token_id.to_le_bytes().as_ref()],
        bump
    )]
    pub nft_info: Account<'info, NFTInfo>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(token_id: u64)]
pub struct BurnNFT<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds = [b"universal_nft_state"], bump)]
    pub universal_nft_state: Account<'info, UniversalNFTState>,

    #[account(
        mut,
        seeds = [b"nft_mint", token_id.to_le_bytes().as_ref()],
        bump
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"nft_info", token_id.to_le_bytes().as_ref()],
        bump
    )]
    pub nft_info: Account<'info, NFTInfo>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(token_id: u64)]
pub struct TransferCrossChain<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"nft_info", token_id.to_le_bytes().as_ref()],
        bump
    )]
    pub nft_info: Account<'info, NFTInfo>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    /// Instructions sysvar for caller verification
    /// CHECK: Instructions sysvar account
    #[account(address = instructions::ID)]
    pub instruction_sysvar: AccountInfo<'info>,
    
    // Gateway accounts for cross-chain transfer
    /// CHECK: Gateway PDA account
    pub gateway_pda: AccountInfo<'info>,
    
    /// CHECK: Whitelist entry for the token
    pub whitelist_entry: AccountInfo<'info>,
    
    /// CHECK: Gateway token account  
    pub gateway_token_account: AccountInfo<'info>,
    
    /// CHECK: Gateway program
    pub gateway_program: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OnCall<'info> {
    #[account(mut, seeds = [b"connected"], bump)]
    pub pda: Account<'info, Pda>,

    #[account(mut)]
    pub pda_ata: Account<'info, TokenAccount>,

    pub mint_account: Account<'info, Mint>,

    #[account(
        init,
        payer = pda,
        space = 8 + size_of::<NFTInfo>(),
        seeds = [b"nft_info", mint_account.key().as_ref()],
        bump
    )]
    pub nft_info: Account<'info, NFTInfo>,

    /// CHECK: Test contract
    pub gateway_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OnRevert<'info> {
    #[account(mut, seeds = [b"connected"], bump)]
    pub pda: Account<'info, Pda>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// Account data structures

#[account]
pub struct UniversalNFTState {
    pub authority: Pubkey,
    pub total_supply: u64,
    pub next_token_id: u64,
}

#[account]
pub struct NFTInfo {
    pub token_id: u64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub is_burned: bool,
    pub cross_chain_data: Option<CrossChainData>,
}

#[account]
pub struct Pda {
    pub last_sender: [u8; 20],
    pub last_message: String,
}

// Cross-chain data structures

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CrossChainNFTTransfer {
    pub token_id: u64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub receiver: Pubkey,
    pub source_chain: Vec<u8>,
}

// Events

#[event]
pub struct NFTMinted {
    pub token_id: u64,
    pub owner: Pubkey,
    pub uri: String,
    pub mint: Pubkey,
}

#[event]
pub struct NFTBurned {
    pub token_id: u64,
    pub owner: Pubkey,
    pub destination_chain: String,
    pub destination_receiver: String,
    pub uri: String,
}

#[event]
pub struct NFTReceived {
    pub token_id: u64,
    pub owner: Pubkey,
    pub uri: String,
    pub from_chain: String,
}

#[event]
pub struct CrossChainTransferInitiated {
    pub token_id: u64,
    pub destination_chain: String,
    pub destination_receiver: String,
    pub gas_amount: u64,
}

// Events
#[event]
pub struct CrossChainTransferEvent {
    pub token_id: u64,
    pub from_chain: String,
    pub to_chain: String,
    pub sender: Pubkey,
    pub receiver: [u8; 20],
}

#[event]
pub struct CrossChainTransferReceived {
    pub token_id: u64,
    pub sender: [u8; 20],
    pub receiver: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[event]
pub struct CrossChainTransferReverted {
    pub token_id: u64,
    pub original_sender: Pubkey,
    pub reverted_amount: u64,
}

// Error codes

#[error_code]
pub enum UniversalNFTError {
    #[msg("Not authorized to perform this action")]
    Unauthorized,
    #[msg("Token ID is already taken")]
    TokenIdTaken,
    #[msg("Not the owner of this NFT")]
    NotOwner,
    #[msg("NFT is already burned")]
    AlreadyBurned,
    #[msg("Invalid token ID")]
    InvalidTokenId,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The data provided could not be converted to a valid UTF-8 string.")]
    InvalidDataFormat,
    #[msg("Failed to decode cross-chain transfer data")]
    DecodingError,
    #[msg("Failed to serialize data")]
    SerializationError,
    #[msg("Not the owner of the NFT")]
    NotOwner,
    #[msg("Invalid caller - must be called by authorized program")]
    InvalidCaller,
}
