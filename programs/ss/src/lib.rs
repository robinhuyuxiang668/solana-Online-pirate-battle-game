pub use crate::errors::SevenSeasError;
use anchor_lang::prelude::*;
pub mod errors;
pub mod state;
pub use state::*;
pub mod instructions;

use anchor_lang::prelude::Account;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use instructions::*;

declare_id!("6Fqvc6LH1put3WbS7CYoWtLYtjzfudQ4ynebkfemKnwe");

/// 击杀玩家奖励: 0.05 SOL
pub const PLAYER_KILL_REWARD: u64 = LAMPORTS_PER_SOL / 20;
/// 宝箱奖励: 0.05 SOL
pub const CHEST_REWARD: u64 = LAMPORTS_PER_SOL / 20;

/// 游戏费用: 0.00 SOL
pub const PLAY_GAME_FEE: u64 = 0;

/// 线程权限PDA的种子
pub const THREAD_AUTHORITY_SEED: &[u8] = b"authority";

#[program]
pub mod ss {

    use super::*;

    /// 初始化游戏账户
    pub fn initialize(_ctx: Context<InitializeAccounts>) -> Result<()> {
        instructions::initialize::initialize(_ctx)
    }

    /// 初始化游戏动作历史账户
    pub fn initialize_game_actions(ctx: Context<InitializeGameActions>) -> Result<()> {
        instructions::initialize_game_actions(ctx)
    }

    /// 初始化游戏数据账户
    pub fn initialize_game_data(ctx: Context<InitializeGameData>) -> Result<()> {
        instructions::initialize_game_data(ctx)
    }

    /// 初始化船只
    pub fn initialize_ship(ctx: Context<InitializeShip>) -> Result<()> {
        instructions::initialize_ship::initialize_ship(ctx)
    }

    /// 升级船只
    pub fn upgrade_ship(ctx: Context<UpgradeShip>) -> Result<()> {
        instructions::upgrade_ship::upgrade_ship(ctx)
    }

    //重置游戏
    pub fn reset(_ctx: Context<Reset>) -> Result<()> {
        _ctx.accounts.game_data_account.load_mut()?.reset()
    }

    //重置船只
    pub fn reset_ship(_ctx: Context<ResetShip>) -> Result<()> {
        _ctx.accounts
            .game_data_account
            .load_mut()?
            .reset_ship(_ctx.accounts.signer.key())
    }

    /// 生成玩家
    pub fn spawn_player(ctx: Context<SpawnPlayer>, avatar: Pubkey) -> Result<()> {
        instructions::spawn_player(ctx, avatar)
    }

    /// 克苏鲁攻击
    pub fn cthulhu(ctx: Context<Cthulhu>, _block_bump: u8) -> Result<()> {
        instructions::cthulhu(ctx)
    }

    /// 射击
    pub fn shoot(ctx: Context<Shoot>) -> Result<()> {
        instructions::shoot(ctx)
    }

    /// 移动玩家(V2)
    pub fn move_player_v2(ctx: Context<MovePlayer>, direction: u8) -> Result<()> {
        instructions::move_player_v2(ctx, direction)
    }
}
