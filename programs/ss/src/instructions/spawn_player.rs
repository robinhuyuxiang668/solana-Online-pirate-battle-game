pub use crate::errors::SevenSeasError;
use crate::{
    ChestVaultAccount, GameDataAccount, Ship, CHEST_REWARD, PLAYER_KILL_REWARD, PLAY_GAME_FEE,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

/// 生成玩家指令处理函数
///
/// # 参数
/// * `ctx` - 指令上下文,包含所有需要的账户信息
/// * `avatar` - 玩家头像公钥,用于标识玩家形象
///
/// # 返回值
/// * `Result<()>` - 成功返回Ok(()),失败返回错误
///
/// # 功能说明
/// 1. 计算玩家拥有的大炮数量和额外生命值
/// 2. 生成玩家并转移游戏费用到宝箱账户
/// 3. 生成宝箱并转移宝箱奖励到宝箱账户
pub fn spawn_player(ctx: Context<SpawnPlayer>, avatar: Pubkey) -> Result<()> {
    // 获取游戏数据账户和船只账户的可变引用,用于后续修改
    let mut game = ctx.accounts.game_data_account.load_mut()?;
    let ship = &mut ctx.accounts.ship;

    // 计算玩家拥有的大炮数量
    // 根据代币精度转换:amount / (10^decimals)
    let decimals = ctx.accounts.cannon_mint.decimals;
    ship.cannons = ctx.accounts.cannon_token_account.amount / 10u64.pow(decimals as u32);

    // 计算额外生命值
    // 根据朗姆酒代币数量:amount / (10^decimals)
    let extra_health = ctx.accounts.rum_token_account.amount / 10u64.pow(decimals as u32);

    msg!("Spawned player! With {} cannons", ship.cannons);

    // 生成玩家,并转移游戏费用到宝箱账户
    // 游戏费用 = 击杀奖励 + 游戏费用
    match game.spawn_player(ctx.accounts.player.to_account_info(), avatar, ship, extra_health) {
        Ok(_) => {
            // 创建CPI上下文,用于转移SOL
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.token_account_owner.to_account_info(),
                    to: ctx.accounts.chest_vault.to_account_info(),
                },
            );
            /*
            加 ?: 如果转账失败，会直接返回错误并退出当前函数
            不加 ?: 需要手动处理成功和失败的情况
            在这个场景中，我们希望转账失败时直接返回错误并终止操作，所以使用 ? 是更好的选择。
            这也符合 Solana 程序的错误处理模式，让错误能够正确地传播给客户端。

             */
            // 转移游戏费用到宝箱账户
            anchor_lang::system_program::transfer(
                cpi_context,
                PLAYER_KILL_REWARD + PLAY_GAME_FEE,
            )?;
        }
        Err(err) => {
            return Err(err.into());
        }
    }

    // 生成宝箱,并转移宝箱奖励到宝箱账户
    match game.spawn_chest(ctx.accounts.player.to_account_info()) {
        Ok(_a) => {
            // 创建CPI上下文,用于转移SOL
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.token_account_owner.to_account_info(),
                    to: ctx.accounts.chest_vault.to_account_info(),
                },
            );
            // 转移宝箱奖励到宝箱账户
            anchor_lang::system_program::transfer(cpi_context, CHEST_REWARD)?;
        }
        Err(err) => {
            return Err(err.into());
        }
    }
    Ok(())
}

#[derive(Accounts)]
pub struct SpawnPlayer<'info> {
    /// CHECK: 这里等于player EOA
    #[account(mut)]
    pub player: AccountInfo<'info>,

    /// 代币账户所有者,这里是玩家EOA账户,需要签名
    /// 用于支付游戏费用和宝箱奖励
    #[account(mut)]
    pub token_account_owner: Signer<'info>,

    /// 宝箱金库账户,用于存储游戏费用和奖励
    #[account(
        mut,
        seeds = [b"chestVault"],
        bump
    )]
    pub chest_vault: Account<'info, ChestVaultAccount>,

    /// 游戏数据账户,存储游戏状态
    /// 包含所有玩家和宝箱的位置信息
    #[account(mut)]
    pub game_data_account: AccountLoader<'info, GameDataAccount>,

    /// 船只账户,存储玩家船只信息
    /// 通过PDA派生,种子为"ship"和NFT账户公钥
    #[account(
        mut,
        seeds = [b"ship", nft_account.key().as_ref()],
        bump
    )]
    pub ship: Account<'info, Ship>,

    /// NFT账户,这里是玩家EOA账户,后续改为代币账户
    /// 用于标识玩家拥有的船只
    /// CHECK: change to token account later
    pub nft_account: AccountInfo<'info>,

    /// 玩家大炮代币账户
    #[account(      
        init_if_needed,
        payer = token_account_owner,
        associated_token::mint = cannon_mint,
        associated_token::authority = token_account_owner      
    )]
    pub cannon_token_account: Account<'info, TokenAccount>,

    /// 大炮代币铸币账户
    pub cannon_mint: Account<'info, Mint>,

    /// 玩家朗姆酒代币账户
    /// 如果不存在则创建,用于存储玩家的朗姆酒代币
    #[account(      
        init_if_needed,
        payer = token_account_owner,
        associated_token::mint = rum_mint,
        associated_token::authority = token_account_owner      
    )]
    pub rum_token_account: Account<'info, TokenAccount>,

    /// 朗姆酒代币铸币账户
    pub rum_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
