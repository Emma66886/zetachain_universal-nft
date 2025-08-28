use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke,
};
use std::mem::size_of;
use anchor_spl::token::{Mint, Token, TokenAccount, MintTo, mint_to, Burn, burn};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata,
};
use mpl_token_metadata::types::DataV2;
use borsh::{BorshDeserialize, BorshSerialize};

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
    pub fn on_call(
        ctx: Context<OnCall>,
        sender: [u8; 20],
        message: Vec<u8>,
    ) -> Result<()> {
        // Decode the NFT transfer data
        let transfer_data = CrossChainNFTTransfer::try_from_slice(&message)
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
}

// Helper function to decode NFT transfer data
fn decode_nft_transfer(data: &[u8]) -> Result<CrossChainNFTTransfer> {
    CrossChainNFTTransfer::try_from_slice(data).map_err(|_| ErrorCode::DecodingError.into())
}
    pub fn on_call(
        ctx: Context<OnCall>,
        amount: u64,
        sender: [u8; 20],
        data: Vec<u8>,
    ) -> Result<()> {
        let pda = &mut ctx.accounts.pda;

        // Store the sender's public key
        pda.last_sender = sender;

        // Try to decode NFT transfer data
        match decode_nft_transfer(&data) {
            Ok(nft_transfer) => {
                // Handle NFT transfer
                let nft_info = &mut ctx.accounts.nft_info;
                
                nft_info.token_id = nft_transfer.token_id;
                nft_info.name = nft_transfer.name.clone();
                nft_info.symbol = nft_transfer.symbol.clone();
                nft_info.uri = nft_transfer.uri.clone();
                nft_info.owner = nft_transfer.receiver;
                nft_info.is_burned = false;
                nft_info.mint = ctx.accounts.mint_account.key();

                // Mint the NFT on Solana
                let cpi_accounts = MintTo {
                    mint: ctx.accounts.mint_account.to_account_info(),
                    to: ctx.accounts.pda_ata.to_account_info(),
                    authority: ctx.accounts.pda.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                
                let seeds = &[b"connected".as_ref(), &[ctx.bumps.pda]];
                let signer = &[&seeds[..]];
                let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
                
                mint_to(cpi_ctx, 1)?;

                emit!(NFTReceived {
                    token_id: nft_transfer.token_id,
                    owner: nft_transfer.receiver,
                    uri: nft_transfer.uri.clone(),
                    from_chain: String::from_utf8_lossy(&nft_transfer.source_chain).to_string(),
                });

                msg!(
                    "NFT transfer executed: token_id {}, receiver {:?}",
                    nft_transfer.token_id,
                    nft_transfer.receiver
                );
            }
            Err(_) => {
                // Fallback to regular message handling
                let message = String::from_utf8(data).map_err(|_| ErrorCode::InvalidDataFormat)?;
                pda.last_message = message;

                if pda.last_message == "sol" {
                    msg!(
                        "On call sol executed with amount {}, sender {:?} and message {}",
                        amount,
                        pda.last_sender,
                        pda.last_message
                    );
                } else {
                    msg!(
                        "On call spl executed with amount {}, spl {:?}, sender {:?} and message {}",
                        amount,
                        ctx.accounts.mint_account,
                        pda.last_sender,
                        pda.last_message
                    );
                }
            }
        }

        Ok(())
    }

    /// Transfer NFT to another chain via ZetaChain Gateway
    pub fn transfer_cross_chain(
        ctx: Context<TransferCrossChain>,
        token_id: u64,
        destination_chain: String,
        destination_receiver: [u8; 20], // ZetaChain EVM address format
        gas_amount: u64,
    ) -> Result<()> {
        let nft_info = &ctx.accounts.nft_info;
        
        // Verify ownership
        require!(nft_info.owner == ctx.accounts.signer.key(), UniversalNFTError::NotOwner);
        require!(!nft_info.is_burned, UniversalNFTError::AlreadyBurned);

        // Create cross-chain transfer data
        let transfer_data = CrossChainNFTTransfer {
            token_id,
            name: nft_info.name.clone(),
            symbol: nft_info.symbol.clone(),
            uri: nft_info.uri.clone(),
            receiver: ctx.accounts.signer.key(), // Use sender's key as placeholder
            source_chain: b"solana".to_vec(),
        };

        // Encode transfer data for cross-chain message
        let encoded_data = transfer_data.try_to_vec().map_err(|_| ErrorCode::SerializationError)?;

        // Create instruction data for ZetaChain Gateway call
        // This follows the pattern from the ZetaChain documentation
        let gateway_instruction_data = GatewayCallInstruction {
            receiver: destination_receiver,
            message: encoded_data,
            gas_limit: gas_amount,
            revert_options: Some(RevertOptions {
                revert_address: ctx.accounts.signer.key(),
                call_on_revert: true,
                abort_address: ctx.accounts.signer.key(),
                revert_message: b"NFT transfer failed".to_vec(),
            }),
        };

        // Serialize the instruction data
        let instruction_data = gateway_instruction_data.try_to_vec()
            .map_err(|_| ErrorCode::SerializationError)?;

        // Create instruction to call ZetaChain Gateway
        let gateway_instruction = Instruction {
            program_id: ctx.accounts.gateway_program.key(),
            accounts: vec![
                AccountMeta::new(ctx.accounts.signer.key(), true),
                AccountMeta::new(ctx.accounts.gateway_pda.key(), false),
                AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
            ],
            data: instruction_data,
        };

        // Execute the cross-chain call via CPI
        invoke(
            &gateway_instruction,
            &[
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.gateway_pda.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Burn the NFT locally after successful gateway call
        let burn_cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let burn_cpi_program = ctx.accounts.token_program.to_account_info();
        let burn_cpi_ctx = CpiContext::new(burn_cpi_program, burn_cpi_accounts);
        burn(burn_cpi_ctx, 1)?;

        // Mark NFT as burned in our state
        let nft_info = &mut ctx.accounts.nft_info;
        nft_info.is_burned = true;

        emit!(CrossChainTransferEvent {
            token_id,
            from_chain: "solana".to_string(),
            to_chain: destination_chain,
            sender: ctx.accounts.signer.key(),
            receiver: destination_receiver,
        });

        Ok(())
    }

// ZetaChain Gateway integration structs
#[derive(BorshSerialize, BorshDeserialize)]
pub struct GatewayCallInstruction {
    pub receiver: [u8; 20],
    pub message: Vec<u8>,
    pub gas_limit: u64,
    pub revert_options: Option<RevertOptions>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct RevertOptions {
    pub revert_address: Pubkey,
    pub call_on_revert: bool,
    pub abort_address: Pubkey,
    pub revert_message: Vec<u8>,
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
    
    /// CHECK: ZetaChain Gateway PDA account
    #[account(mut)]
    pub gateway_pda: AccountInfo<'info>,

    /// CHECK: ZetaChain Gateway program for cross-chain transfers
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
}

#[account]
pub struct Pda {
    pub last_sender: [u8; 20],
    pub last_message: String,
}

// Cross-chain data structures

#[derive(BorshSerialize, BorshDeserialize, Clone)]
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
}
