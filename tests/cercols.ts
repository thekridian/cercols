import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Cercols } from "../target/types/cercols";
import { expect } from "chai";
import {
  createSignerFromKeypair,
  generateSigner,
  percentAmount,
  publicKey,
  signerIdentity,
  sol,
  some,
} from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  MPL_TOKEN_METADATA_PROGRAM_ID,
  createNft,
  createProgrammableNft,
  findMasterEditionPda,
  findMetadataPda,
  findTokenRecordPda,
  mplTokenMetadata,
  verifyCollectionV1,
} from "@metaplex-foundation/mpl-token-metadata";
import { readFileSync } from "fs";
import path from "path";
import {
  SPL_SYSTEM_PROGRAM_ID,
  SPL_TOKEN_PROGRAM_ID,
  findAssociatedTokenPda,
} from "@metaplex-foundation/mpl-toolbox";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";

describe("cercols", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Cercols as Program<Cercols>;

  const umi = createUmi(provider.connection.rpcEndpoint).use(
    mplTokenMetadata()
  );

  // Umi needs the keypair from disk
  const keyFileContents = JSON.parse(
    readFileSync(
      path.join(process.env.HOME, ".config/solana/id.json")
    ).toString()
  );
  const signer = umi.eddsa.createKeypairFromSecretKey(
    new Uint8Array(keyFileContents)
  );
  umi.use(signerIdentity(createSignerFromKeypair(umi, signer)));

  // Programs we need
  const tokenProgram = new anchor.web3.PublicKey(SPL_TOKEN_PROGRAM_ID);
  const metadataProgram = new anchor.web3.PublicKey(
    MPL_TOKEN_METADATA_PROGRAM_ID
  );
  const associatedTokenProgram = new anchor.web3.PublicKey(
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  const systemProgram = new anchor.web3.PublicKey(SPL_SYSTEM_PROGRAM_ID);
  const sysvarInstructions = new anchor.web3.PublicKey(
    anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY
  );

  // Collection Mint
  const collectionMint = generateSigner(umi);
  const collectionMintPubkey = new anchor.web3.PublicKey(
    collectionMint.publicKey
  );

  // NFT of the collection - must be owned by the Signer
  const nftMint2 = generateSigner(umi);
  const nftMint2Pubkey = new anchor.web3.PublicKey(nftMint2.publicKey);

  const nftToken = findAssociatedTokenPda(umi, {
    mint: nftMint2.publicKey,
    owner: umi.identity.publicKey,
  });
  const nftTokenPubkey = new anchor.web3.PublicKey(publicKey(nftToken));

  const nftMetadata = findMetadataPda(umi, { mint: nftMint2.publicKey });
  const nftMetadataPubkey = new anchor.web3.PublicKey(publicKey(nftMetadata));

  const nftEdition = findMasterEditionPda(umi, { mint: nftMint2.publicKey });
  const nftEditionPubkey = new anchor.web3.PublicKey(publicKey(nftEdition));

  const [poolPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("cercols_pool"),
      collectionMintPubkey.toBuffer(),
      program.provider.publicKey.toBytes(),
    ],
    program.programId
  );

  const [nftAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("nft_authority"), poolPda.toBytes()],
    program.programId
  );

  const nftCustody = getAssociatedTokenAddressSync(
    nftMint2Pubkey,
    nftAuthorityPda,
    true,
    tokenProgram,
    associatedTokenProgram
  );
  // findAssociatedTokenPda(umi, {
  //   mint: nftMint2.publicKey,
  //   owner: publicKey(nftAuthorityPda),
  // });
  // const nftCustodyPubkey = new anchor.web3.PublicKey(nftCustody);

  const sourceTokenRecord = findTokenRecordPda(umi, {
    mint: nftMint2.publicKey,
    token: publicKey(nftToken),
  });
  const sourceTokenRecordPubkey = new anchor.web3.PublicKey(
    publicKey(sourceTokenRecord)
  );

  const destinationTokenRecord = findTokenRecordPda(umi, {
    mint: nftMint2.publicKey,
    token: publicKey(nftCustody),
  });
  const destinationTokenRecordPubkey = new anchor.web3.PublicKey(
    publicKey(destinationTokenRecord)
  );

  const swapFeeLamports = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL);

  before(async () => {
    console.log("Airdropping SOL...");

    await umi.rpc.airdrop(umi.payer.publicKey, sol(10));

    console.log("Creating collection NFT...");

    await createNft(umi, {
      mint: collectionMint,
      name: "Cercols Collection",
      uri: "https://cercols/collection",
      sellerFeeBasisPoints: percentAmount(5),
      isCollection: true,
    }).sendAndConfirm(umi);

    console.log("Creating pNFT...");
    await createProgrammableNft(umi, {
      mint: nftMint2,
      tokenOwner: umi.identity.publicKey,
      name: "Cercols #1",
      uri: "https://cercols/1",
      sellerFeeBasisPoints: percentAmount(2),
      collection: some({ key: collectionMint.publicKey, verified: false }),
    }).sendAndConfirm(umi);

    console.log("Verifying collection...");
    await verifyCollectionV1(umi, {
      metadata: nftMetadata,
      collectionMint: collectionMint.publicKey,
      authority: umi.payer,
    }).sendAndConfirm(umi);
  });

  // after(async () => {});

  it("Is initialized!", async () => {
    await program.methods
      .initPool(swapFeeLamports)
      .accounts({
        collectionMint: collectionMintPubkey,
        pool: poolPda,
        nftAuthority: nftAuthorityPda,
      })
      .rpc();

    const account = await program.account.poolState.fetch(poolPda);

    expect(account.collectionMint.toBase58()).to.eq(
      collectionMintPubkey.toBase58()
    );
    expect(account.authority.toBase58()).to.eq(
      anchor.getProvider().publicKey.toBase58()
    );
    expect(account.swapFeeLamports.toNumber()).to.eq(
      swapFeeLamports.toNumber()
    );
    expect(account.size).to.eq(0);
  });

  it("Can deposit an NFT from the collection", async () => {
    // console.log("pool: ", poolPda.toString());
    // console.log("nftAuthority: ", nftAuthorityPda.toString());
    // console.log("nftMint: ", nftMint2Pubkey.toString());
    // console.log("nftToken: ", nftTokenPubkey.toString());
    // console.log("user: ", provider.wallet.publicKey.toString());
    // console.log("nftMetadata: ", nftMetadataPubkey.toString());
    // console.log("nftEdition: ", nftEditionPubkey.toString());
    // console.log("nftCustody: ", nftCustody.toString());
    // console.log("sourceTokenRecord: ", sourceTokenRecordPubkey.toString());
    // console.log(
    //   "destinationTokenRecord: ",
    //   destinationTokenRecordPubkey.toString()
    // );
    // console.log("tokenProgram: ", tokenProgram.toString());
    // console.log("metadataProgram: ", metadataProgram.toString());
    // console.log("associatedTokenProgram: ", associatedTokenProgram.toString());
    // console.log("systemProgram: ", systemProgram.toString());
    // console.log("sysvarInstructions: ", sysvarInstructions.toString());

    // const tx = await program.methods
    //   .deposit()
    //   .accounts({
    //     pool: poolPda,
    //     nftAuthority: nftAuthorityPda,
    //     nftMint: nftMint2Pubkey,
    //     nftToken: nftTokenPubkey,
    //     user: provider.wallet.publicKey,
    //     nftMetadata: nftMetadataPubkey,
    //     nftEdition: nftEditionPubkey,
    //     nftCustody: nftCustody,
    //     sourceTokenRecord: sourceTokenRecordPubkey,
    //     destinationTokenRecord: destinationTokenRecordPubkey,
    //     tokenProgram,
    //     metadataProgram,
    //     associatedTokenProgram,
    //     systemProgram,
    //     sysvarInstructions,
    //   })
    //   .rpc({ skipPreflight: true });
    // console.log("TX: ", tx);

    try {
      const tx = await program.methods
        .deposit()
        .accounts({
          pool: poolPda,
          nftAuthority: nftAuthorityPda,
          nftMint: nftMint2Pubkey,
          nftToken: nftTokenPubkey,
          user: provider.wallet.publicKey,
          nftMetadata: nftMetadataPubkey,
          nftEdition: nftEditionPubkey,
          nftCustody: nftCustody,
          sourceTokenRecord: sourceTokenRecordPubkey,
          destinationTokenRecord: destinationTokenRecordPubkey,
          tokenProgram,
          metadataProgram,
          associatedTokenProgram,
          systemProgram,
          sysvarInstructions,
        })
        .rpc();
      console.log("TX: ", tx);
    } catch (error) {
      console.log("error", error);
    }
  });
});
