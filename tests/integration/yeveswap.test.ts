import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MathUtil, PDA } from "@orca-so/common-sdk";
import { PriceMath } from "@orca-so/whirlpools-sdk";
import BN from "bn.js";
import { IDL } from "../../target/types/yeveswap";
import Decimal from "decimal.js";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  Connection,
  Commitment,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
  transfer,
} from "@solana/spl-token";
import * as assert from "assert";

const PDA_YEVEPOOL_SEED = "yevepool";
const PDA_POSITION_SEED = "position";
const PDA_METADATA_SEED = "metadata";
const PDA_TICK_ARRAY_SEED = "tick_array";
const PDA_FEE_TIER_SEED = "fee_tier";
const PDA_ORACLE_SEED = "oracle";
const PDA_POSITION_BUNDLE_SEED = "position_bundle";
const PDA_BUNDLED_POSITION_SEED = "bundled_position";

const TICK_ARRAY_SIZE = 88;

export const METADATA_PROGRAM_ADDRESS = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

const tokenMintAKey = new PublicKey(
  "7vEpiNkomzeF2uDw8uuDFqEcQfaWbpPgmFf41G5Y7W4o"
);
const tokenMintBKey = new PublicKey(
  "CU1f67B7n3XzwbHkFvciuH6Yqe8kiaEFfSZHzLNRvtYi"
);
const rewardMint = new PublicKey(
  "JC7EAyPpZKjt5bAQj7a3zpMRwsac5AxMoY5DHnPMffJr"
);
// admin wallet's account for rewardMint
const adminRewardMintAccount = new PublicKey(
  "FxBmrwYsMG9YjqsdZrKRZvqr4wNXHj8pA2WarsAM5dUC"
);

export enum TickSpacing {
  One = 1,
  Stable = 8,
  ThirtyTwo = 32,
  SixtyFour = 64,
  Standard = 128,
}

function wait(milliseconds: number) {
  return new Promise((resolve) => {
    setTimeout(resolve, milliseconds);
  });
}

const wallet = NodeWallet.local();

export type InitConfigParams = {
  yevepoolsConfigKeypair: Keypair;
  feeAuthority: PublicKey;
  collectProtocolFeesAuthority: PublicKey;
  rewardEmissionsSuperAuthority: PublicKey;
  defaultProtocolFeeRate: number;
  funder: PublicKey;
};

export interface TestYevepoolsConfigKeypairs {
  feeAuthorityKeypair: Keypair;
  collectProtocolFeesAuthorityKeypair: Keypair;
  rewardEmissionsSuperAuthorityKeypair: Keypair;
}

const configKeypairs: TestYevepoolsConfigKeypairs = {
  feeAuthorityKeypair: Keypair.generate(),
  collectProtocolFeesAuthorityKeypair: Keypair.generate(),
  rewardEmissionsSuperAuthorityKeypair: Keypair.generate(),
};

const configInitInfo = {
  yevepoolsConfigKeypair: Keypair.generate(),
  feeAuthority: configKeypairs.feeAuthorityKeypair.publicKey,
  collectProtocolFeesAuthority:
    configKeypairs.collectProtocolFeesAuthorityKeypair.publicKey,
  rewardEmissionsSuperAuthority:
    configKeypairs.rewardEmissionsSuperAuthorityKeypair.publicKey,
  defaultProtocolFeeRate: 300,
  funder: wallet.publicKey,
};

const tokenVaultAKeypair = Keypair.generate();
const tokenVaultBKeypair = Keypair.generate();
const rewardVaultKeypair = Keypair.generate();

let initializedConfigInfo: InitConfigParams;

export function getAssociatedTokenAddressSync(
  mint: PublicKey,
  owner: PublicKey,
  allowOwnerOffCurve = false,
  programId = TOKEN_PROGRAM_ID,
  associatedTokenProgramId = ASSOCIATED_TOKEN_PROGRAM_ID
): PublicKey {
  if (!allowOwnerOffCurve && !PublicKey.isOnCurve(owner.toBuffer())) {
    console.log("TOken owner offcurve");
    return;
  }

  const [address] = PublicKey.findProgramAddressSync(
    [owner.toBuffer(), programId.toBuffer(), mint.toBuffer()],
    associatedTokenProgramId
  );

  return address;
}

describe("yeveswap contract test", () => {
  // Configure the client to use the local cluster.
  // Use Mainnet-fork for testing
  const commitment: Commitment = "confirmed";
  const connection = new Connection(
    "https://virulent-wandering-reel.solana-devnet.quiknode.pro/c29fc55807faf8297c2fed73d3cd98150fd970ad/",
    {
      commitment,
      // wsEndpoint: "wss://api.devnet.solana.com/",
    }
  );

  const options = anchor.AnchorProvider.defaultOptions();
  const provider = new anchor.AnchorProvider(connection, wallet, options);

  anchor.setProvider(provider);

  const programId = new PublicKey(
    "2QE2SSJJvLdFAUkCzKkmX3UmiKsFbJ9qn8mmufeokx6p" // contract address
  ); // call this contract in this test script
  const program = new anchor.Program(IDL, programId, provider);

  it("Check balance", async () => {
    let balance = await connection.getBalance(wallet.publicKey);
    console.log(
      `${wallet.publicKey.toString()} has ${balance / 1000000000} SOL`
    );
    // const fundingTx = new Transaction();
    // fundingTx.add(
    //   SystemProgram.transfer({
    //     fromPubkey: wallet.publicKey,
    //     toPubkey: configKeypairs.feeAuthorityKeypair.publicKey,
    //     lamports: 100000000,
    //   })
    // );
    // let txhash = await provider.sendAndConfirm(fundingTx);
  });

  it("Initialize config!", async () => {
    // call init config function
    await program.methods
      .initializeConfig(
        configInitInfo.feeAuthority,
        configInitInfo.collectProtocolFeesAuthority,
        configInitInfo.rewardEmissionsSuperAuthority,
        configInitInfo.defaultProtocolFeeRate
      )
      .accounts({
        config: configInitInfo.yevepoolsConfigKeypair.publicKey,
        funder: configInitInfo.funder,
        systemProgram: SystemProgram.programId,
      })
      .signers([configInitInfo.yevepoolsConfigKeypair])
      .rpc();
    wait(6000);
    const configAccount = await program.account.yevepoolsConfig.fetch(
      configInitInfo.yevepoolsConfigKeypair.publicKey
    );

    assert.ok(
      configAccount.collectProtocolFeesAuthority.equals(
        configInitInfo.collectProtocolFeesAuthority
      )
    );
    assert.ok(configAccount.feeAuthority.equals(configInitInfo.feeAuthority));

    assert.ok(
      configAccount.rewardEmissionsSuperAuthority.equals(
        configInitInfo.rewardEmissionsSuperAuthority
      )
    );

    assert.equal(
      configAccount.defaultProtocolFeeRate,
      configInitInfo.defaultProtocolFeeRate
    );
    initializedConfigInfo = configInitInfo;
  });

  it("fail on passing in already initialized yevepool account", async () => {
    await assert.rejects(
      program.methods
        .initializeConfig(
          initializedConfigInfo.feeAuthority,
          initializedConfigInfo.collectProtocolFeesAuthority,
          initializedConfigInfo.rewardEmissionsSuperAuthority,
          initializedConfigInfo.defaultProtocolFeeRate
        )
        .accounts({
          config: initializedConfigInfo.yevepoolsConfigKeypair.publicKey,
          funder: initializedConfigInfo.funder,
          systemProgram: SystemProgram.programId,
        })
        .signers([initializedConfigInfo.yevepoolsConfigKeypair])
        .rpc(),
      /0x0/
    );
  });

  it("init fee tier", async () => {
    // init fee tier function
    const feeTierPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(PDA_FEE_TIER_SEED),
        configInitInfo.yevepoolsConfigKeypair.publicKey.toBuffer(),
        new BN(TickSpacing.Standard).toArrayLike(Buffer, "le", 2),
      ],
      programId
    );

    await program.methods
      .initializeFeeTier(TickSpacing.Standard, 800)
      .accounts({
        config: configInitInfo.yevepoolsConfigKeypair.publicKey,
        feeTier: feeTierPda[0],
        feeAuthority: configInitInfo.feeAuthority,
        funder: configInitInfo.funder,
        systemProgram: SystemProgram.programId,
      })
      .signers([configKeypairs.feeAuthorityKeypair])
      .rpc();
    wait(6000);
    const feeTierAccount = await program.account.feeTier.fetch(feeTierPda[0]);

    assert.ok(feeTierAccount.tickSpacing == TickSpacing.Standard);
    assert.ok(feeTierAccount.defaultFeeRate == 800);
  });

  it("initialize pool", async () => {
    const price = PriceMath.tickIndexToSqrtPriceX64(0);
    const yevepoolPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(PDA_YEVEPOOL_SEED),
        configInitInfo.yevepoolsConfigKeypair.publicKey.toBuffer(),
        tokenMintAKey.toBuffer(),
        tokenMintBKey.toBuffer(),
        new BN(TickSpacing.Standard).toArrayLike(Buffer, "le", 2),
      ],
      programId
    );

    const feeTierPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(PDA_FEE_TIER_SEED),
        configInitInfo.yevepoolsConfigKeypair.publicKey.toBuffer(),
        new BN(TickSpacing.Standard).toArrayLike(Buffer, "le", 2),
      ],
      programId
    );

    const yevepoolBump = {
      yevepoolBump: yevepoolPda[1],
    };

    // return;
    await program.methods
      .initializePool(yevepoolBump, TickSpacing.Standard, price)
      .accounts({
        yevepoolsConfig: configInitInfo.yevepoolsConfigKeypair.publicKey,
        tokenMintA: tokenMintAKey,
        tokenMintB: tokenMintBKey,
        funder: configInitInfo.funder,
        yevepool: yevepoolPda[0],
        tokenVaultA: tokenVaultAKeypair.publicKey,
        tokenVaultB: tokenVaultBKeypair.publicKey,
        feeTier: feeTierPda[0],
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([wallet.payer, tokenVaultAKeypair, tokenVaultBKeypair])
      .rpc();
    wait(6000);
    const feeTierAccount = await program.account.feeTier.fetch(feeTierPda[0]);

    assert.ok(feeTierAccount.tickSpacing == TickSpacing.Standard);
    assert.ok(feeTierAccount.defaultFeeRate == 800);

    const yevepoolAccount = await program.account.yevepool.fetch(
      yevepoolPda[0]
    );

    console.log("Original yevepool", yevepoolAccount);
  });

  it("initialize_tick_array", async () => {
    const tickSpacing = TickSpacing.Standard;
    const tickLowerIndex = -1280,
      tickUpperIndex = 1280;

    const startTickLowerIndex =
      Math.floor(tickLowerIndex / tickSpacing / TICK_ARRAY_SIZE) *
      tickSpacing *
      TICK_ARRAY_SIZE;

    const startTickUpperIndex =
      Math.floor(tickUpperIndex / tickSpacing / TICK_ARRAY_SIZE) *
      tickSpacing *
      TICK_ARRAY_SIZE;

    const yevepoolPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(PDA_YEVEPOOL_SEED),
        configInitInfo.yevepoolsConfigKeypair.publicKey.toBuffer(),
        tokenMintAKey.toBuffer(),
        tokenMintBKey.toBuffer(),
        new BN(TickSpacing.Standard).toArrayLike(Buffer, "le", 2),
      ],
      programId
    );

    let tickArrayPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(PDA_TICK_ARRAY_SEED),
        yevepoolPda[0].toBuffer(),
        Buffer.from(startTickLowerIndex.toString()),
      ],
      programId
    );

    await program.methods
      .initializeTickArray(startTickLowerIndex)
      .accounts({
        yevepool: yevepoolPda[0],
        tickArray: tickArrayPda[0],
        funder: configInitInfo.funder,
        systemProgram: SystemProgram.programId,
      })
      .signers([])
      .rpc();
    wait(6000);
    let tickArrayAccount = await program.account.tickArray.fetch(
      tickArrayPda[0]
    );

    assert.ok(tickArrayAccount.startTickIndex == startTickLowerIndex);
    assert.ok(
      tickArrayAccount.yevepool.toString() == yevepoolPda[0].toString()
    );

    tickArrayPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(PDA_TICK_ARRAY_SEED),
        yevepoolPda[0].toBuffer(),
        Buffer.from(startTickUpperIndex.toString()),
      ],
      programId
    );

    await program.methods
      .initializeTickArray(startTickUpperIndex)
      .accounts({
        yevepool: yevepoolPda[0],
        tickArray: tickArrayPda[0],
        funder: configInitInfo.funder,
        systemProgram: SystemProgram.programId,
      })
      .signers([])
      .rpc();

    tickArrayAccount = await program.account.tickArray.fetch(tickArrayPda[0]);

    assert.ok(tickArrayAccount.startTickIndex == startTickUpperIndex);
    assert.ok(
      tickArrayAccount.yevepool.toString() == yevepoolPda[0].toString()
    );
  });

  // it("initialize_reward", async () => {
  //   const yevepoolPda = PublicKey.findProgramAddressSync(
  //     [
  //       Buffer.from(PDA_YEVEPOOL_SEED),
  //       configInitInfo.yevepoolsConfigKeypair.publicKey.toBuffer(),
  //       tokenMintAKey.toBuffer(),
  //       tokenMintBKey.toBuffer(),
  //       new BN(TickSpacing.Standard).toArrayLike(Buffer, "le", 2),
  //     ],
  //     programId
  //   );

  //   await program.methods
  //     .initializeReward(0) // rewardIndex = 0
  //     .accounts({
  //       rewardAuthority:
  //         configKeypairs.rewardEmissionsSuperAuthorityKeypair.publicKey,
  //       yevepool: yevepoolPda[0],
  //       funder: configInitInfo.funder,
  //       rewardMint,
  //       rewardVault: rewardVaultKeypair.publicKey,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     })
  //     .signers([
  //       rewardVaultKeypair,
  //       configKeypairs.rewardEmissionsSuperAuthorityKeypair,
  //     ])
  //     .rpc();
  //   wait(6000);
  //   const yevepoolAccount = await program.account.yevepool.fetch(
  //     yevepoolPda[0]
  //   );

  //   assert.ok(
  //     yevepoolAccount.rewardInfos[0].mint.toString() == rewardMint.toString()
  //   );
  //   assert.ok(
  //     yevepoolAccount.rewardInfos[0].vault.toString() ==
  //       rewardVaultKeypair.publicKey.toString()
  //   );
  // });

  // it("set_reward_emissions", async () => {
  //   const emissionsPerSecondX64 = new anchor.BN(10_000)
  //     .shln(16)
  //     .div(new anchor.BN(60 * 60 * 24));
  //   const rewardIndex = 0;

  //   const yevepoolPda = PublicKey.findProgramAddressSync(
  //     [
  //       Buffer.from(PDA_YEVEPOOL_SEED),
  //       configInitInfo.yevepoolsConfigKeypair.publicKey.toBuffer(),
  //       tokenMintAKey.toBuffer(),
  //       tokenMintBKey.toBuffer(),
  //       new BN(TickSpacing.Standard).toArrayLike(Buffer, "le", 2),
  //     ],
  //     programId
  //   );

  //   // send tokens to rewardVault
  //   const rewardVaultTokenAccount = await createAccount(
  //     connection,
  //     wallet.payer,
  //     rewardMint,
  //     rewardVaultKeypair.publicKey
  //   );

  //   await transfer(
  //     connection,
  //     wallet.payer,
  //     adminRewardMintAccount,
  //     rewardVaultTokenAccount,
  //     wallet.publicKey,
  //     10000 * 1e9 // decimal 9
  //   );
  //   wait(6000);

  //   await program.methods
  //     .setRewardEmissions(rewardIndex, emissionsPerSecondX64) // rewardIndex = 0
  //     .accounts({
  //       rewardAuthority:
  //         configKeypairs.rewardEmissionsSuperAuthorityKeypair.publicKey,
  //       yevepool: yevepoolPda[0],
  //       rewardVault: rewardVaultKeypair.publicKey,
  //     })
  //     .signers([configKeypairs.rewardEmissionsSuperAuthorityKeypair])
  //     .rpc();
  //   wait(6000);
  //   const yevepoolAccount = await program.account.yevepool.fetch(
  //     yevepoolPda[0]
  //   );

  //   assert.ok(
  //     yevepoolAccount.rewardInfos[0].mint.toString() == rewardMint.toString()
  //   );
  //   assert.ok(
  //     yevepoolAccount.rewardInfos[0].vault.toString() ==
  //       rewardVaultKeypair.publicKey.toString()
  //   );
  // });

  it("open_position", async () => {
    const tickSpacing = TickSpacing.Standard;
    const tickLowerIndex = -1280,
      tickUpperIndex = 1280;

    const positionMintKeypair = Keypair.generate();
    const positionPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(PDA_POSITION_SEED),
        positionMintKeypair.publicKey.toBuffer(),
      ],
      programId
    );
    const metadataPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(PDA_METADATA_SEED),
        METADATA_PROGRAM_ADDRESS.toBuffer(),
        positionMintKeypair.publicKey.toBuffer(),
      ],
      METADATA_PROGRAM_ADDRESS
    );

    const positionTokenAccountAddress = getAssociatedTokenAddressSync(
      positionMintKeypair.publicKey,
      wallet.publicKey
    );

    const yevepoolPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(PDA_YEVEPOOL_SEED),
        configInitInfo.yevepoolsConfigKeypair.publicKey.toBuffer(),
        tokenMintAKey.toBuffer(),
        tokenMintBKey.toBuffer(),
        new BN(TickSpacing.Standard).toArrayLike(Buffer, "le", 2),
      ],
      programId
    );

    const bumps = {
      positionBump: positionPda[1],
    };

    await program.methods
      .openPosition(bumps, tickLowerIndex, tickUpperIndex) // rewardIndex = 0
      .accounts({
        funder: configInitInfo.funder,
        owner: configInitInfo.funder,
        position: positionPda[0],
        positionMint: positionMintKeypair.publicKey,
        positionTokenAccount: positionTokenAccountAddress,
        yevepool: yevepoolPda[0],
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([positionMintKeypair])
      .rpc();
    wait(6000);

    const startTickLowerIndex =
      Math.floor(tickLowerIndex / tickSpacing / TICK_ARRAY_SIZE) *
      tickSpacing *
      TICK_ARRAY_SIZE;

    const startTickUpperIndex =
      Math.floor(tickUpperIndex / tickSpacing / TICK_ARRAY_SIZE) *
      tickSpacing *
      TICK_ARRAY_SIZE;

    const tickLowerArrayPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(PDA_TICK_ARRAY_SEED),
        yevepoolPda[0].toBuffer(),
        Buffer.from(startTickLowerIndex.toString()),
      ],
      programId
    );

    const tickUpperArrayPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(PDA_TICK_ARRAY_SEED),
        yevepoolPda[0].toBuffer(),
        Buffer.from(startTickUpperIndex.toString()),
      ],
      programId
    );

    const liquidityAmount = new anchor.BN(100_000_000_000);
    const tokenMaxA = new anchor.BN(70_000_000_000);
    const tokenMaxB = new anchor.BN(30_000_000_000);

    const tokenOwnerAccountA = getAssociatedTokenAddressSync(
      tokenMintAKey,
      wallet.publicKey
    );

    const tokenOwnerAccountB = getAssociatedTokenAddressSync(
      tokenMintBKey,
      wallet.publicKey
    );

    await program.methods
      .increaseLiquidity(liquidityAmount, tokenMaxA, tokenMaxB) // rewardIndex = 0
      .accounts({
        yevepool: yevepoolPda[0],
        position: positionPda[0],
        positionTokenAccount: positionTokenAccountAddress,
        positionAuthority: wallet.publicKey,
        tokenOwnerAccountA,
        tokenOwnerAccountB,
        tokenVaultA: tokenVaultAKeypair.publicKey,
        tokenVaultB: tokenVaultBKeypair.publicKey,
        tickArrayLower: tickLowerArrayPda[0],
        tickArrayUpper: tickUpperArrayPda[0],
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([])
      .rpc();

    const yevepoolAccount = await program.account.yevepool.fetch(
      yevepoolPda[0]
    );

    const positionAccount = await program.account.position.fetch(
      positionPda[0]
    );

    console.log(yevepoolAccount, positionAccount);
  });
});
