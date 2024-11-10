pub use crate::errors::SevenSeasError;
use crate::Ship;
use anchor_lang::prelude::Account;
use anchor_lang::prelude::*;

pub fn initialize_ship(ctx: Context<InitializeShip>) -> Result<()> {
    msg!("Ship Initialized!");
    // 设置初始生命值
    ctx.accounts.new_ship.health = 50;
    ctx.accounts.new_ship.start_health = 50;
    // 设置初始等级
    ctx.accounts.new_ship.level = 1;
    // 设置初始升级次数
    ctx.accounts.new_ship.upgrades = 0;
    // 设置初始炮台数量
    ctx.accounts.new_ship.cannons = 1;
    Ok(())
}

/// 初始化船只所需账户
#[derive(Accounts)]
pub struct InitializeShip<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// 船只账户
    #[account(
        init,
        payer = signer, 
        seeds = [b"ship", nft_account.key().as_ref()],
        bump,
        space = 1024
    )]
    pub new_ship: Account<'info, Ship>,
    /// NFT账户
    /// CHECK:
    pub nft_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
