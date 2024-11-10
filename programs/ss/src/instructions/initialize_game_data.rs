use crate::GameDataAccount;
use anchor_lang::prelude::*;

pub fn initialize_game_data(_ctx: Context<InitializeGameData>) -> Result<()> {
    msg!("Game Data Account Initialized!");
    // 初始化游戏数据账户
    let game_data = &mut _ctx.accounts.new_game_data_account.load_init()?;
    // 初始化棋盘
    **game_data = GameDataAccount::default();
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeGameData<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // 游戏数据账户 - 用于存储游戏状态
    #[account(
        init,
        payer = signer,
        seeds = [b"level"],
        bump,
        space = 10240
    )]
    pub new_game_data_account: AccountLoader<'info, GameDataAccount>,

    pub system_program: Program<'info, System>,
}
