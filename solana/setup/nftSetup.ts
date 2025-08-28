import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { Connected } from "../target/types/connected";
import fs from "fs";
import path from "path";

const METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

interface NFTSetupConfig {
  programId: PublicKey;
  authority: Keypair;
  connection: anchor.web3.Connection;
  program: Program<Connected>;
}

export class SolanaNFTSetup {
  private config: NFTSetupConfig;

  constructor(config: NFTSetupConfig) {
    this.config = config;
  }

  /**
   * Initialize the Universal NFT state
   */
  async initializeNFTState(): Promise<PublicKey> {
    const [universalNFTState] = PublicKey.findProgramAddressSync(
      [Buffer.from("universal_nft_state")],
      this.config.programId
    );

    const [pda] = PublicKey.findProgramAddressSync(
      [Buffer.from("connected")],
      this.config.programId
    );

    try {
      await this.config.program.methods
        .initialize()
        .accounts({
          signer: this.config.authority.publicKey,
          universalNftState: universalNFTState,
          pda: pda,
          systemProgram: SystemProgram.programId,
        })
        .signers([this.config.authority])
        .rpc();

      console.log("✅ Universal NFT state initialized");
      console.log("📍 State account:", universalNFTState.toString());
      
      return universalNFTState;
    } catch (error) {
      console.error("❌ Error initializing NFT state:", error);
      throw error;
    }
  }

  /**
   * Mint a new Universal NFT
   */
  async mintNFT(
    tokenId: number,
    name: string,
    symbol: string,
    uri: string,
    to: PublicKey
  ): Promise<{
    mint: PublicKey;
    tokenAccount: PublicKey;
    nftInfo: PublicKey;
  }> {
    const [universalNFTState] = PublicKey.findProgramAddressSync(
      [Buffer.from("universal_nft_state")],
      this.config.programId
    );

    const [mint] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_mint"), Buffer.from(tokenId.toString().padStart(8, "0"))],
      this.config.programId
    );

    const [nftInfo] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_info"), Buffer.from(tokenId.toString().padStart(8, "0"))],
      this.config.programId
    );

    const tokenAccount = await getOrCreateAssociatedTokenAccount(
      this.config.connection,
      this.config.authority,
      mint,
      to,
      true
    );

    const [metadata] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      METADATA_PROGRAM_ID
    );

    try {
      await this.config.program.methods
        .mintNft(
          new anchor.BN(tokenId),
          name,
          symbol,
          uri,
          to
        )
        .accounts({
          signer: this.config.authority.publicKey,
          universalNftState: universalNFTState,
          mint: mint,
          tokenAccount: tokenAccount.address,
          nftInfo: nftInfo,
          metadata: metadata,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          metadataProgram: METADATA_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([this.config.authority])
        .rpc();

      console.log("✅ NFT minted successfully");
      console.log("🎨 Token ID:", tokenId);
      console.log("🔗 Mint:", mint.toString());
      console.log("📄 Metadata URI:", uri);

      return {
        mint,
        tokenAccount: tokenAccount.address,
        nftInfo,
      };
    } catch (error) {
      console.error("❌ Error minting NFT:", error);
      throw error;
    }
  }

  /**
   * Burn an NFT for cross-chain transfer
   */
  async burnNFT(
    tokenId: number,
    destinationChain: string,
    destinationReceiver: string
  ): Promise<void> {
    const [universalNFTState] = PublicKey.findProgramAddressSync(
      [Buffer.from("universal_nft_state")],
      this.config.programId
    );

    const [mint] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_mint"), Buffer.from(tokenId.toString().padStart(8, "0"))],
      this.config.programId
    );

    const [nftInfo] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_info"), Buffer.from(tokenId.toString().padStart(8, "0"))],
      this.config.programId
    );

    const tokenAccount = await getOrCreateAssociatedTokenAccount(
      this.config.connection,
      this.config.authority,
      mint,
      this.config.authority.publicKey,
      true
    );

    try {
      await this.config.program.methods
        .burnNft(
          new anchor.BN(tokenId),
          destinationChain,
          destinationReceiver
        )
        .accounts({
          signer: this.config.authority.publicKey,
          universalNftState: universalNFTState,
          mint: mint,
          tokenAccount: tokenAccount.address,
          nftInfo: nftInfo,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([this.config.authority])
        .rpc();

      console.log("✅ NFT burned for cross-chain transfer");
      console.log("🎨 Token ID:", tokenId);
      console.log("🌉 Destination:", destinationChain);
      console.log("👤 Receiver:", destinationReceiver);
    } catch (error) {
      console.error("❌ Error burning NFT:", error);
      throw error;
    }
  }

  /**
   * Get NFT information
   */
  async getNFTInfo(tokenId: number): Promise<any> {
    const [nftInfo] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_info"), Buffer.from(tokenId.toString().padStart(8, "0"))],
      this.config.programId
    );

    try {
      const info = await this.config.program.account.nftInfo.fetch(nftInfo);
      return {
        tokenId: info.tokenId.toString(),
        name: info.name,
        symbol: info.symbol,
        uri: info.uri,
        owner: info.owner.toString(),
        mint: info.mint.toString(),
        isBurned: info.isBurned,
      };
    } catch (error) {
      console.error("❌ Error fetching NFT info:", error);
      return null;
    }
  }
}

// Export helper functions
export const setupSolanaNFTProgram = async (): Promise<SolanaNFTSetup> => {
  // Set up Anchor provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Load program
  const programId = new PublicKey("9BjVGjn28E58LgSi547JYEpqpgRoo1TErkbyXiRSNDQy");
  const program = anchor.workspace.Connected as Program<Connected>;

  // Load authority keypair
  const authorityPath = path.resolve(process.env.HOME || "", ".config/solana/id.json");
  const authorityKeypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(authorityPath, "utf8")))
  );

  const setup = new SolanaNFTSetup({
    programId,
    authority: authorityKeypair,
    connection: provider.connection,
    program,
  });

  return setup;
};
