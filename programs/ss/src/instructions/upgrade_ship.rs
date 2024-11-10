pub use crate::errors::SevenSeasError;
use anchor_lang::prelude::*;

use anchor_lang::prelude::Account;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

use crate::{Ship, TOKEN_DECIMAL_MULTIPLIER};

/// 升级船只
pub fn upgrade_ship(ctx: Context<UpgradeShip>) -> Result<()> {
    // 创建代币转账指令
    let transfer_instruction = Transfer {
        from: ctx.accounts.player_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    // 创建CPI上下文
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
    );

    // 根据升级次数确定升级费用和新属性
    let cost: u64;
    match ctx.accounts.new_ship.upgrades {
        0 => {
            ctx.accounts.new_ship.health = 100;
            ctx.accounts.new_ship.upgrades = 1;
            cost = 5; // 5代币
        }
        1 => {
            ctx.accounts.new_ship.health = 150;
            ctx.accounts.new_ship.upgrades = 2;
            cost = 200; // 200代币
        }
        2 => {
            ctx.accounts.new_ship.health = 300;
            ctx.accounts.new_ship.upgrades = 3;
            cost = 1500; // 1500代币
        }
        3 => {
            ctx.accounts.new_ship.health = 500;
            ctx.accounts.new_ship.upgrades = 4;
            cost = 25000; // 25000代币
        }
        _ => {
            return Err(SevenSeasError::MaxShipLevelReached.into());
        }
    }
    // 执行代币转账,乘以小数位数
    anchor_spl::token::transfer(cpi_ctx, cost * TOKEN_DECIMAL_MULTIPLIER);

    msg!("Ship upgraded to level: {}", ctx.accounts.new_ship.upgrades);

    Ok(())
}

/// 升级船只所需账户
#[derive(Accounts)]
pub struct UpgradeShip<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// 船只账户
    #[account(
        seeds = [b"ship", nft_account.key().as_ref()],
        bump
    )]
    #[account(mut)]
    pub new_ship: Account<'info, Ship>,
    /// NFT账户
    /// CHECK:
    #[account(mut)]
    pub nft_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    /// 玩家代币账户
    #[account( 
        mut,     
        associated_token::mint = mint_of_token_being_sent,
        associated_token::authority = signer      
    )]
    pub player_token_account: Account<'info, TokenAccount>,
    /// 代币金库账户
    #[account(
        mut,
        seeds=[b"token_vault".as_ref(), mint_of_token_being_sent.key().as_ref()],
        token::mint=mint_of_token_being_sent,
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub mint_of_token_being_sent: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}
