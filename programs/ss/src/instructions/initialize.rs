// 导入必要的依赖
pub use crate::errors::SevenSeasError;
use crate::ChestVaultAccount;
use anchor_lang::prelude::Account;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

pub fn initialize(_ctx: Context<InitializeAccounts>) -> Result<()> {
    msg!("Initialized!");
    Ok(())
}

/*
AccountLoader:
适用于大型账户（通常大于 10KB=10240字节）的情况 ，在你的代码中
GameDataAccount 使用 AccountLoader 是因为它的 space 设置为 10240 字节（10KB）

提供了零拷贝反序列化（zero-copy deserialization）功能 ,数据GameDataAccount 用修饰:#[account(zero_copy(unsafe))]
通过 load() 和 load_mut() 方法访问数据，可以减少内存使用


Box<Account>:智能指针,数据放到堆上
适用于小型账户，
直接将整个账户数据加载到内存中
使用更简单，访问数据更方便
在你的代码中，ChestVaultAccount 和 GameActionHistory 使用 Box<Account> 是因为它们相对较小
适用于编译时大小未知的数据，如 game_actions：GameActionHistory，有Vec<GameAction>大小未知
当需要将数据从栈移动到堆以延长生命周期时

选择建议
当账户数据大于 10KB 时，推荐使用 AccountLoader
当账户数据较小，且需要频繁访问时，使用 Box<Account>
如果关注性能和内存使用，大型数据结构应该使用 AccountLoader
如果追求开发便利性，且数据量小，使用 Box<Account>

Box
数据会被完整复制
需要额外的堆内存分配
适合小型数据结构（<10KB）
立即加载，数据会直接加载到堆内存中

AccountLoader
零拷贝访问
无需额外堆内存
适合大型数据结构（>10KB）
延迟加载，只有在调用 load() 或 load_mut() 时才会真正加载数据


如果不使用 AccountLoader 或 Box，处理 Account 数据时确实可能会遇到问题：
#[account]
pub struct LargeAccount {
    pub data: [u8; 20000]  // 比如这是一个很大的数组
}

pub fn risky_handler(ctx: Context<Handler>) -> Result<()> {
    // 危险！这会尝试将整个数据结构复制到栈上
    let account = &ctx.accounts.large_account;

    // 这种操作可能导致栈溢出
    let data_copy = account.data;
}

安全的访问方式
pub fn safe_handler(ctx: Context<Handler>) -> Result<()> {
    // 安全：只获取引用，不复制数据
    let account = &ctx.accounts.large_account;

    // 通过引用访问数据
    let data_ref = &account.data;

    // 如果确实需要修改，使用可变引用
    let mut account = &mut ctx.accounts.large_account;
    account.data[0] = 1;
}


可能出现的问题：
栈溢出
如果 Account 数据结构很大，直接复制会超过 10KB 栈限制
特别是在结构体包含大数组或嵌套结构时

性能问题
不必要的数据复制会降低性能
增加内存使用

并发访问问题
可能在多个地方同时修改数据
缺少 AccountLoader 提供的借用检查机制

建议的最佳实践：
大数据结构（>10KB）：
使用 AccountLoader + #[account(zero_copy)]
避免整体复制，使用引用访问
中等大小数据：
使用 Box<Account>
或者谨慎使用引用
小数据结构：
可以直接使用，但仍建议通过引用访问
避免不必要的复制

*/
#[derive(Accounts)]
pub struct InitializeAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // 宝箱金库账户 - 用于存储和发放奖励SOL
    #[account(
        init,
        seeds = [b"chestVault"],
        bump,
        payer = signer,
        space = 8
    )]
    pub chest_vault: Box<Account<'info, ChestVaultAccount>>,

    // 代币账户所有者PDA
    /// CHECK: 派生的PDAs
    #[account(
        init,
        payer = signer,
        seeds=[b"token_account_owner_pda".as_ref()],
        bump,
        space = 8
    )]
    token_account_owner_pda: AccountInfo<'info>,

    /*

    1. **当前的 PDA 方式**:
    #[account(
        init,
        payer = signer,
        seeds=[b"token_vault".as_ref(), mint_of_token_being_sent.key().as_ref()],
        token::mint=mint_of_token_being_sent,
        token::authority=token_account_owner_pda,
        bump
    )]
    vault_token_account: Account<'info, TokenAccount>,
    ```
    - 账户所有者：我们的游戏程序
    - 代币账户控制权（authority）：token_account_owner_pda


    2. **如果改为 ATA**:
    ```rust
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint_of_token_being_sent,
        associated_token::authority = token_account_owner_pda,
    )]
    vault_token_account: Account<'info, TokenAccount>,

    - 账户所有者：Associated Token Program
    - 代币账户控制权（authority）：token_account_owner_pda

    这就是为什么在需要程序完全控制的场景下（比如游戏金库），我们倾向于使用 PDA 而不是 ATA。因为：
    PDA 让我们的程序拥有账户的完全控制权
    我们可以通过程序进行 CPI（Cross-Program Invocation）来管理这些账户
    提供了更灵活的权限控制机制
         */
    // 代币金库账户 - 用于存储游戏中的代币
    #[account(
        init,
        payer = signer,
        seeds=[b"token_vault".as_ref(), mint_of_token_being_sent.key().as_ref()],
        token::mint=mint_of_token_being_sent,
        token::authority=token_account_owner_pda,
        bump
    )]
    vault_token_account: Account<'info, TokenAccount>,

    // 将要发送的代币的铸币账户
    pub mint_of_token_being_sent: Account<'info, Mint>,

    // 系统程序
    pub system_program: Program<'info, System>,

    // 代币程序
    pub token_program: Program<'info, Token>,
}
