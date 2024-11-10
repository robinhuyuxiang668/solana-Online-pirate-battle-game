use crate::GameActionHistory;
use anchor_lang::prelude::*;

pub fn initialize_game_actions(_ctx: Context<InitializeGameActions>) -> Result<()> {
    msg!("Game Actions Account Initialized!");
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeGameActions<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // 游戏动作历史账户
    #[account(
        init,
        payer = signer,
        seeds = [b"gameActions_history"],
        bump,
        space = 8 + GameActionHistory::INIT_SPACE
    )]
    pub game_actions: Account<'info, GameActionHistory>,

    pub system_program: Program<'info, System>,
}
