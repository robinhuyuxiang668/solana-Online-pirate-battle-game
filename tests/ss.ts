// 导入必要的依赖
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";

import { Ss } from "../target/types/ss";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getMint,
  mintTo,
  getAccount,
  getOrCreateAssociatedTokenAccount,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import fs from "fs";
import { Keypair } from "@solana/web3.js";
import {
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
  LAMPORTS_PER_SOL,
  PublicKey,
} from "@solana/web3.js";

// 定义代币铸币地址
// let goldTokenMint = new anchor.web3.PublicKey(
//   "goLdQwNaZToyavwkbuPJzTt5XPNR3H7WQBGenWtzPH3"
// );
// let cannonTokenMint = new anchor.web3.PublicKey(
//   "boomkN8rQpbgGAKcWvR3yyVVkjucNYcq7gTav78NQAG"
// );
// let rumTokenMint = new anchor.web3.PublicKey(
//   "rumwqxXmjKAmSdkfkc5qDpHTpETYJRyXY22DWYUmWDt"
// );
let goldTokenMintKeypair = Keypair.generate();
let goldTokenMint = goldTokenMintKeypair.publicKey;
let cannonTokenMintKeypair = Keypair.generate();
let cannonTokenMint = cannonTokenMintKeypair.publicKey;
let rumTokenMintKeypair = Keypair.generate();
let rumTokenMint = rumTokenMintKeypair.publicKey;
let tokenOwnerKeypair = Keypair.generate();

let payer: anchor.Wallet;

describe("ss", () => {
  // 配置客户端使用本地集群
  const provider = anchor.AnchorProvider.env();

  // 根据网络设置commitment
  const commitment = provider.connection.rpcEndpoint.includes("devnet")
    ? "confirmed"
    : provider.connection.commitment;
  const connection = new anchor.web3.Connection(
    provider.connection.rpcEndpoint,
    commitment
  );
  const newProvider = new anchor.AnchorProvider(
    connection,
    provider.wallet,
    provider.opts
  );
  anchor.setProvider(newProvider);

  const program = anchor.workspace.Ss as Program<Ss>;
  const player = anchor.web3.Keypair.generate();
  payer = provider.wallet as anchor.Wallet;

  // 所有者密钥

  console.log("player 地址是: ", player.publicKey.toBase58());

  it("初始化!", async () => {
    let confirmOptions = {
      skipPreflight: true,
    };

    await transfer(player.publicKey, 2 * LAMPORTS_PER_SOL);
    await transfer(tokenOwnerKeypair.publicKey, 2 * LAMPORTS_PER_SOL);

    // 创建金币代币
    const gold_mint = await createMint(
      anchor.getProvider().connection,
      tokenOwnerKeypair,
      tokenOwnerKeypair.publicKey,
      tokenOwnerKeypair.publicKey,
      9,
      goldTokenMintKeypair
    );

    // 创建大炮代币
    const cannon_mint = await createMint(
      anchor.getProvider().connection,
      tokenOwnerKeypair,
      tokenOwnerKeypair.publicKey,
      tokenOwnerKeypair.publicKey,
      9,
      cannonTokenMintKeypair
    );

    // 创建朗姆酒代币
    const rum_mint = await createMint(
      anchor.getProvider().connection,
      tokenOwnerKeypair,
      tokenOwnerKeypair.publicKey,
      tokenOwnerKeypair.publicKey,
      9, // 使用9位小数以完全匹配CLI小数默认值
      rumTokenMintKeypair
    );

    // 查找代币账户所有者PDA
    let [tokenAccountOwnerPda, bump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("token_account_owner_pda", "utf8")],
        program.programId
      );

    // 查找代币金库PDA
    let [token_vault, bump2] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("token_vault", "utf8"), gold_mint.toBuffer()],
        program.programId
      );

    console.log("铸币地址: " + gold_mint.toBase58());

    // 获取铸币信息
    let mintInfo = await getMint(anchor.getProvider().connection, gold_mint);
    console.log("铸币供应量" + mintInfo.supply.toString());
    const mintDecimals = Math.pow(10, mintInfo.decimals);

    // 为玩家创建代币账户
    const playerGoldTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      player,
      gold_mint,
      tokenOwnerKeypair.publicKey
    );

    const playerCannonTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      player,
      cannon_mint,
      tokenOwnerKeypair.publicKey
    );

    const playerRumTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      player,
      rum_mint,
      tokenOwnerKeypair.publicKey
    );

    console.log("发送者代币账户: " + playerGoldTokenAccount.address.toBase58());
    console.log("金库账户: " + token_vault);
    console.log("代币账户所有者PDA: " + tokenAccountOwnerPda);

    // 向玩家铸造金币
    const mintToPlayerResult = await mintTo(
      anchor.getProvider().connection,
      player,
      gold_mint,
      playerGoldTokenAccount.address,
      tokenOwnerKeypair,
      10000000 * mintDecimals // 1000万代币,9位小数
    );
    console.log("铸币签名: " + mintToPlayerResult);

    // 向玩家铸造大炮代币
    const mintCannonsToPlayerResult = await mintTo(
      anchor.getProvider().connection,
      player,
      cannon_mint,
      playerCannonTokenAccount.address,
      tokenOwnerKeypair,
      10000000 * mintDecimals // 1000万代币,9位小数
    );
    console.log("铸币签名: " + mintCannonsToPlayerResult);

    // 向玩家铸造朗姆酒代币
    const mintRumToPlayerResult = await mintTo(
      anchor.getProvider().connection,
      player,
      rum_mint,
      playerRumTokenAccount.address,
      tokenOwnerKeypair,
      10000000 * mintDecimals // 1000万代币,9位小数
    );

    console.log("铸币签名: " + mintRumToPlayerResult);

    // 获取玩家金币账户信息
    let tokenAccountInfo = await getAccount(
      anchor.getProvider().connection,
      playerGoldTokenAccount.address
    );
    console.log(
      "玩家拥有的金币数量: " + tokenAccountInfo.amount / BigInt(mintDecimals)
    );

    console.log("玩家金币: " + (10000000 * mintDecimals).toString());

    // 查找游戏相关PDA
    const [level] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("level")],
      program.programId
    );

    const [chestVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("chestVault")],
      program.programId
    );

    const [gameActions] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("gameActions_history")],
      program.programId
    );

    // 初始化游戏
    const tx = await program.methods
      .initialize()
      .accounts({
        signer: player.publicKey,
        // newGameDataAccount: level,
        chestVault: chestVault,
        //gameActions: gameActions,
        tokenAccountOwnerPda: tokenAccountOwnerPda,
        vaultTokenAccount: token_vault,
        mintOfTokenBeingSent: gold_mint,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([player])
      .rpc();
    console.log("initialize 交易签名", tx);

    // 初始化游戏动作历史账户
    const initGameActionsTx = await program.methods
      .initializeGameActions()
      .accounts({
        signer: player.publicKey,
        gameActions: gameActions,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player])
      .rpc();
    console.log("initialize game actions 交易签名", initGameActionsTx);

    // 初始化游戏数据账户
    const initGameDataTx = await program.methods
      .initializeGameData()
      .accounts({
        signer: player.publicKey,
        newGameDataAccount: level,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player])
      .rpc();
    console.log("initialize game data 交易签名", initGameDataTx);

    // 向程序代币金库铸造代币
    const mintToProgramResult = await mintTo(
      anchor.getProvider().connection,
      player,
      gold_mint,
      token_vault,
      tokenOwnerKeypair,
      10000000 * mintDecimals // 1000万代币,9位小数
    );
    console.log("铸币签名: " + mintToProgramResult);
  });

  it("初始化船只!", async () => {
    let confirmOptions = {
      skipPreflight: true,
    };

    await transfer(player.publicKey, 0.1 * LAMPORTS_PER_SOL);

    // 查找船只PDA
    const [shipPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("ship"), player.publicKey.toBuffer()],
      program.programId
    );

    // 查找代币金库
    let [token_vault, bump2] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("token_vault", "utf8"), goldTokenMint.toBuffer()],
        program.programId
      );

    // 创建玩家代币账户
    const playerTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      player,
      goldTokenMint,
      player.publicKey
    );

    // 获取铸币信息
    let mintInfo = await getMint(
      anchor.getProvider().connection,
      goldTokenMint
    );
    console.log("铸币供应量" + mintInfo.supply.toString());
    const mintDecimals = Math.pow(10, mintInfo.decimals);

    // 向玩家铸造代币
    const mintToPlayerResult = await mintTo(
      anchor.getProvider().connection,
      player,
      goldTokenMint,
      playerTokenAccount.address,
      tokenOwnerKeypair,
      1000000 * mintDecimals, // 100万代币,9位小数
      [],
      confirmOptions
    );
    console.log("铸币签名: " + mintToPlayerResult);
    await anchor
      .getProvider()
      .connection.confirmTransaction(mintToPlayerResult, "confirmed");

    // 获取玩家代币账户信息
    let tokenAccountInfo = await getAccount(
      anchor.getProvider().connection,
      playerTokenAccount.address
    );
    console.log(
      "玩家拥有的代币数量: " + tokenAccountInfo.amount / BigInt(mintDecimals)
    );

    console.log("玩家代币: " + (10000000 * mintDecimals).toString());

    // 初始化船只
    let tx = await program.methods
      .initializeShip()
      .accounts({
        newShip: shipPDA,
        signer: player.publicKey,
        nftAccount: player.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player])
      .rpc();
    console.log("初始化船只交易", tx);

    // 升级船只
    tx = await program.methods
      .upgradeShip()
      .accounts({
        newShip: shipPDA,
        signer: player.publicKey,
        nftAccount: player.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        vaultTokenAccount: token_vault,
        mintOfTokenBeingSent: goldTokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        playerTokenAccount: playerTokenAccount.address,
      })
      .signers([player])
      .rpc();
    console.log("升级船只交易", tx);

    tx = await program.methods
      .upgradeShip()
      .accounts({
        newShip: shipPDA,
        signer: player.publicKey,
        nftAccount: player.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        vaultTokenAccount: token_vault,
        mintOfTokenBeingSent: goldTokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        playerTokenAccount: playerTokenAccount.address,
      })
      .signers([player])
      .rpc();
    console.log("升级船只交易", tx);
  });

  it("生成船只!", async () => {
    await transfer(player.publicKey, 0.1 * LAMPORTS_PER_SOL);

    // 查找游戏相关PDA
    const [level] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("level")],
      program.programId
    );

    const [chestVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("chestVault")],
      program.programId
    );
    const avatarPubkey = anchor.web3.Keypair.generate();

    const [shipPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("ship"), player.publicKey.toBuffer()],
      program.programId
    );

    // 创建玩家大炮代币账户
    const playerCannonTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      player,
      cannonTokenMint,
      player.publicKey
    );

    console.log("玩家大炮账户: " + playerCannonTokenAccount.address.toString());

    // 创建玩家朗姆酒代币账户
    const playerRumTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      player,
      rumTokenMint,
      player.publicKey
    );
    console.log("玩家朗姆酒账户: " + playerRumTokenAccount.address.toString());

    // 生成玩家
    const tx = await program.methods
      .spawnPlayer(avatarPubkey.publicKey)
      .accounts({
        player: player.publicKey,
        tokenAccountOwner: player.publicKey,
        gameDataAccount: level,
        chestVault: chestVault,
        nftAccount: player.publicKey,
        ship: shipPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
        cannonTokenAccount: playerCannonTokenAccount.address,
        cannonMint: cannonTokenMint,
        rumTokenAccount: playerRumTokenAccount.address,
        rumMint: rumTokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([player]);
    let result = await tx.rpc();
    console.log("交易签名", result);
  });

  it("移动!", async () => {
    let confirmOptions = {
      skipPreflight: true,
    };

    await transfer(player.publicKey, 0.1 * LAMPORTS_PER_SOL);

    // 查找游戏相关PDA
    const [level] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("level")],
      program.programId
    );

    const [chestVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("chestVault")],
      program.programId
    );

    let [tokenAccountOwnerPda, bump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("token_account_owner_pda", "utf8")],
        program.programId
      );

    let [token_vault, bump2] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("token_vault", "utf8"), goldTokenMint.toBuffer()],
        program.programId
      );

    const [gameActions] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("gameActions_history")],
      program.programId
    );

    // 创建玩家代币账户
    const playerTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      player,
      goldTokenMint,
      player.publicKey
    );

    // 移动玩家
    const tx = await program.methods
      .movePlayerV2(2)
      .accounts({
        player: player.publicKey,
        gameDataAccount: level,
        chestVault: chestVault,
        tokenAccountOwner: player.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenAccountOwnerPda: tokenAccountOwnerPda,
        vaultTokenAccount: token_vault,
        playerTokenAccount: playerTokenAccount.address,
        mintOfTokenBeingSent: goldTokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        gameActions: gameActions,
      })
      .signers([player])
      .rpc();
    console.log("交易签名", tx);
  });

  it("射击!", async () => {
    let confirmOptions = {
      skipPreflight: process.env.NODE_ENV === "test", // 在测试中使用 confirmed 提供更好的可靠性
    };

    await transfer(player.publicKey, 0.1 * LAMPORTS_PER_SOL);

    // 查找游戏相关PDA
    const [level] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("level")],
      program.programId
    );

    const [chestVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("chestVault")],
      program.programId
    );

    const [gameActions] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("gameActions_history")],
      program.programId
    );
    let [tokenAccountOwnerPda, bump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("token_account_owner_pda", "utf8")],
        program.programId
      );

    let [token_vault, bump2] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("token_vault", "utf8"), goldTokenMint.toBuffer()],
        program.programId
      );

    // 创建玩家代币账户
    const playerTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      player,
      goldTokenMint,
      player.publicKey
    );

    // 射击
    const tx = await program.methods
      .shoot(0)
      .accounts({
        player: player.publicKey,
        tokenAccountOwner: player.publicKey,
        gameDataAccount: level,
        chestVault: chestVault,
        gameActions: gameActions,
        tokenAccountOwnerPda: tokenAccountOwnerPda,
        vaultTokenAccount: token_vault,
        playerTokenAccount: playerTokenAccount.address,
        mintOfTokenBeingSent: goldTokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([player])
      .rpc();
    console.log("交易签名", tx);
  });

  it("克苏鲁!", async () => {
    await transfer(player.publicKey, 0.1 * LAMPORTS_PER_SOL);

    let confirmOptions = {
      skipPreflight: true,
    };

    // 查找游戏相关PDA
    const [level] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("level")],
      program.programId
    );

    const [chestVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("chestVault")],
      program.programId
    );

    const [gameActions] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("gameActions_history")],
      program.programId
    );

    let [tokenAccountOwnerPda, bump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("token_account_owner_pda", "utf8")],
        program.programId
      );

    let [token_vault, bump2] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("token_vault", "utf8"), goldTokenMint.toBuffer()],
        program.programId
      );

    // 创建玩家代币账户
    const playerTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      player,
      goldTokenMint,
      player.publicKey
    );

    // 克苏鲁
    const tx = await program.methods
      .cthulhu(0)
      .accounts({
        player: player.publicKey,
        tokenAccountOwner: player.publicKey,
        gameDataAccount: level,
        chestVault: chestVault,
        gameActions: gameActions,
        tokenAccountOwnerPda: tokenAccountOwnerPda,
        vaultTokenAccount: token_vault,
        playerTokenAccount: playerTokenAccount.address,
        mintOfTokenBeingSent: goldTokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([player])
      .rpc();
    console.log("交易签名", tx);
  });

  async function transfer(toAddress: PublicKey, amount: number) {
    // 创建转账交易
    const transaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: toAddress,
        lamports: amount,
      })
    );

    // 发送并确认交易
    const txSignature = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [payer.payer]
    );

    // 验证接收地址的余额
    let receiverBalance = await provider.connection.getBalance(toAddress);

    console.log(`接收地址余额: ${receiverBalance / LAMPORTS_PER_SOL} SOL`);

    return txSignature;
  }
});
