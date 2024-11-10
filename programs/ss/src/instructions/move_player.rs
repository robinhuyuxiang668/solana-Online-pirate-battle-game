pub use crate::errors::SevenSeasError;
use crate::{GameActionHistory, GameDataAccount};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

/// 移动玩家的指令处理函数
///
/// # 参数
/// * `ctx` - 指令上下文,包含所有需要的账户
/// * `direction` - 移动方向,0-3分别代表上下左右
///
/// # 返回值
/// * `Result<()>` - 成功返回Ok(()),失败返回错误
pub fn move_player_v2(ctx: Context<MovePlayer>, direction: u8) -> Result<()> {
    // 获取游戏数据账户的可变引用
    let game = &mut ctx.accounts.game_data_account.load_mut()?;

    // 调用游戏逻辑处理移动
    match game.move_in_direction(
        direction,
        ctx.accounts.player.to_account_info(),
        ctx.accounts.chest_vault.to_account_info(),
        ctx.accounts.vault_token_account.to_account_info(),
        ctx.accounts.player_token_account.to_account_info(),
        ctx.accounts.token_account_owner_pda.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.bumps.token_account_owner_pda,
        &mut ctx.accounts.game_actions,
    ) {
        Ok(_val) => {}
        Err(err) => {
            panic!("Error: {}", err);
        }
    }
    // 打印游戏状态
    game.print();
    Ok(())
}

#[derive(Accounts)]
pub struct MovePlayer<'info> {
    /// 宝箱金库账户,用于存储和发放奖励
    /// CHECK: 这是一个简单的SOL账户,不需要额外检查
    #[account(mut)]
    pub chest_vault: AccountInfo<'info>,

    /// 游戏数据账户,存储游戏状态
    #[account(mut)]
    pub game_data_account: AccountLoader<'info, GameDataAccount>,

    /// 玩家账户,必须是签名者
    #[account(mut)]
    pub player: Signer<'info>,

    /// CHECK: 这里等于player EOA
    pub token_account_owner: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    /// 玩家的代币账户,用于接收奖励代币
    #[account(      
        init_if_needed,
        payer = player,
        associated_token::mint = mint_of_token_being_sent,
        associated_token::authority = token_account_owner      
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    /// 代币金库账户,用于存储游戏中的代币
    #[account(
        init_if_needed,
        payer = player,
        seeds=[b"token_vault".as_ref(), mint_of_token_being_sent.key().as_ref()],
        token::mint=mint_of_token_being_sent,
        token::authority=token_account_owner_pda,
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// 代币账户所有者PDA
    /// CHECK: 这是一个派生的PDA账户
    #[account(
        mut,
        seeds=[b"token_account_owner_pda".as_ref()],
        bump
    )]
    pub token_account_owner_pda: AccountInfo<'info>,

    /// 将要发送的代币的铸币账户
    pub mint_of_token_being_sent: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// 游戏动作历史账户,用于记录游戏中的动作
    #[account(mut)]
    pub game_actions: Account<'info, GameActionHistory>,
}
