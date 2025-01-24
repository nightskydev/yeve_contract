import * as anchor from "@coral-xyz/anchor";
import { Idl } from "@coral-xyz/anchor";
import { splTokenProgram } from "@project-serum/spl-token";
import {
  PublicKey,
  SystemProgram,
  Transaction,
  Connection,
  Commitment,
  Keypair,
} from "@solana/web3.js";
import {
  getAssociatedTokenAddressSync,
  createTransferInstruction,
  createAssociatedTokenAccountInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  fetchMetadata,
  findMetadataPda,
  mplTokenMetadata,
} from "@metaplex-foundation/mpl-token-metadata";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { Pda, publicKey } from "@metaplex-foundation/umi";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import idl from "./idl.json";
import { TOKEN_PROGRAM_ID,TOKEN_2022_PROGRAM_ID, getMint } from "@solana/spl-token";
import { constants } from "./test/constants";

export function wait(milliseconds: number) {
  return new Promise((resolve) => {
    setTimeout(resolve, milliseconds);
  });
}

const signerKeypair = anchor.web3.Keypair.fromSecretKey(new Uint8Array([104,12,19,47,108,61,23,35,239,153,127,75,227,161,87,84,90,245,218,29,68,11,63,105,171,227,83,214,145,126,234,178,18,59,85,219,8,139,180,119,30,179,82,193,78,74,19,91,190,72,226,41,57,11,129,116,175,247,215,185,154,74,173,236]));
const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", {
  commitment,
  // wsEndpoint: "wss://api.devnet.solana.com/",
});
const options = anchor.AnchorProvider.defaultOptions();

const wallet = new NodeWallet(signerKeypair);

const provider = new anchor.AnchorProvider(connection, wallet, options);

anchor.setProvider(provider);

// CAUTTION: if you are intended to use the program that is deployed by yourself,
// please make sure that the programIDs are consistent
const program = new anchor.Program(idl as Idl, new PublicKey("5kZULJfKLcHw9UbnTpPxmzWfnZk5ZRgsFDUiggjqdKzY"), provider);

const addRewardPool = async () => {
  if (!program.provider.publicKey) return;

  // Determined Seeds
  const adminSeed = "admin";
  const userSeed = "user"

  const adminKey = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode(adminSeed)),
    ],
    program.programId
  )[0];

  console.log(adminKey)

  const userInfo = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode(userSeed)),
      wallet.publicKey.toBuffer(),
    ],
    program.programId
  )[0];
  console.log({userInfo})

  const tokenMint = Keypair.generate()
  const [nftTokenAccount] = PublicKey.findProgramAddressSync(
    [wallet.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), tokenMint.publicKey.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  const metadataProgram = new PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );
  const [nftMetadataAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      metadataProgram.toBuffer(),
      tokenMint.publicKey.toBuffer(),
    ],
    metadataProgram
  );
  const metadataUpdateAuth = new PublicKey("3axbTs2z5GBy6usVbNVoqEgZMng3vZvMnAoX29BFfwhr")

  const testGenesisNft = new PublicKey("ChDZ4zkMGbY36DJjWWa4TWQX3F1kKQUcmDGpfd3A5g3B");
  const [genesisNftTokenAccount] = PublicKey.findProgramAddressSync(
    [wallet.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), testGenesisNft.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  const [testNftMetadataAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      metadataProgram.toBuffer(),
      testGenesisNft.toBuffer(),
    ],
    metadataProgram
  );
  console.log(testNftMetadataAccount.toBase58())

  // await program.methods
  //   .initialize(new anchor.BN(1000), "HHHHHHHHHH", "SSSSSSSS", "https://raw.githubusercontent.com/solana-developers/professional-education/main/labs/sample-nft-offchain-data.json")
  //   .accounts({
  //     signer: wallet.publicKey,
  //     state: adminKey,
  //     systemProgram: anchor.web3.SystemProgram.programId,
  //   })
  //   .signers([wallet.payer])
  //   .rpc({ commitment: 'confirmed', preflightCommitment: 'processed' });

  // await program.methods
  //   .updatePresaleInfo(new anchor.BN(1737607908 + 3600), "My NFT", "", "https://raw.githubusercontent.com/solana-developers/professional-education/main/labs/sample-nft-offchain-data.json", new anchor.BN(17998))
  //   .accounts({
  //     signer: wallet.publicKey,
  //     state: adminKey,
  //     collection: new PublicKey("AYhZWHDD21vKpHXLeF3hZ6VraoYLd9Nj8Phk8BA9ipLy"),
  //     genesisCollection: new PublicKey("933DNvtpge6YrU3abRX3RJtkjxPungad24zSB8sV8j4X"),
  //     systemProgram: anchor.web3.SystemProgram.programId,
  //   })
  //   .signers([wallet.payer])
  //   .rpc({ commitment: 'confirmed', preflightCommitment: 'processed' });

    // await program.methods
    // .addToWhitelist()
    // .accounts({
    //   signer: wallet.publicKey,
    //   userInfo,
    //   systemProgram: anchor.web3.SystemProgram.programId,
    // })
    // .signers([wallet.payer])
    // .rpc({ commitment: 'confirmed', preflightCommitment: 'processed' });

  await program.methods
    .mint()
    .accounts({
      signer: wallet.publicKey,
      state: adminKey,
      userInfo,
      tokenMint: tokenMint.publicKey,
      nftTokenAccount,
      metadataProgram,
      nftMetadataAccount,
      metadataUpdateAuth,
      genesisNftMetadataAccount: testNftMetadataAccount,
      genesisNftTokenAccount,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID
    })
    .signers([wallet.payer, tokenMint])
    .rpc({ commitment: 'confirmed', preflightCommitment: 'processed' });

  await wait(500);
  console.log("Minted NFT", tokenMint.publicKey.toBase58());

  const fetchedAdminState: any = await program.account.state.fetch(
    adminKey
  );
  console.log({ fetchedAdminState });

  // const fetchedUserState: any = await program.account.userInfo.fetch(
  //   userInfo
  // );
  // console.log({ fetchedUserState });
};

addRewardPool();
