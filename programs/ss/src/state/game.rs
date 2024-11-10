// 导入错误处理模块
pub use crate::errors::SevenSeasError;
// 导入游戏相关常量
use crate::PLAYER_KILL_REWARD;
use crate::{Ship, CHEST_REWARD};
// 导入 Anchor 相关模块
use anchor_lang::prelude::*;
use anchor_spl::token::Transfer;

// 游戏棋盘大小常量
const BOARD_SIZE_X: usize = 10;
const BOARD_SIZE_Y: usize = 10;

// 棋盘格子状态常量
const STATE_EMPTY: u8 = 0; // 空格子
const STATE_PLAYER: u8 = 1; // 玩家所在格子
const STATE_CHEST: u8 = 2; // 宝箱所在格子

// 游戏动作类型常量
const GAME_ACTION_SHIP_SHOT: u8 = 0; // 船只射击
const GAME_ACTION_SHIP_TAKEN_DAMAGE: u8 = 1; // 船只受伤
const GAME_ACTION_SHIP_CTHULUH_ATTACKED_SHIP: u8 = 2; // 克苏鲁攻击船只
const GAME_ACTION_SHIP_COINS_COLLECTED: u8 = 3; // 收集金币

// 游戏奖励常量
const CHEST_COIN_REWARD: u64 = 10; // 宝箱奖励金币数
const DESTROY_SHIP_COIN_REWARD: u64 = 10; // 摧毁船只奖励金币数

// 代币精度乘数
pub const TOKEN_DECIMAL_MULTIPLIER: u64 = 1000000000;

// 重置游戏账户结构
#[derive(Accounts)]
pub struct Reset<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    //签名者账户
    #[account(
        mut,
        seeds = [b"level"],
        bump,
    )]
    pub game_data_account: AccountLoader<'info, GameDataAccount>, // 游戏数据账户
}

// 重置船只账户结构
#[derive(Accounts)]
pub struct ResetShip<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // 签名者账户
    #[account(
        mut,
        seeds = [b"level"],
        bump,
    )]
    pub game_data_account: AccountLoader<'info, GameDataAccount>, // 游戏数据账户
}

/*
GameDataAccount 结构体定义
使用了以下属性:
- zero_copy: 使用零拷贝反序列化,直接从程序账户内存中读取数据,提高性能
- unsafe: 不进行内存安全检查
- packed: 数据按字节对齐
- repr(packed): 以最紧凑的方式布局结构体内存,不添加填充字节
- Default: 允许使用默认值初始化结构体

这些属性的组合在 Solana 程序中很常见,因为:
1. 需要高效的内存访问(zero_copy)
2. 需要精确的内存布局(packed)
3. 需要能够创建默认实例(Default)

对于存储游戏状态的 BoardAccount 特别重要,因为它需要:
1. 高效访问游戏数据
2. 在链上存储时节省空间
3. 能够方便地初始化新的游戏状态
*/
#[account(zero_copy(unsafe))]
#[repr(packed)]
#[derive(Default)]
pub struct GameDataAccount {
    board: [[Tile; BOARD_SIZE_X]; BOARD_SIZE_Y], // 游戏棋盘数组
    action_id: u64,                              // 动作ID计数器
}

// 棋盘格子结构体
#[zero_copy(unsafe)]
#[repr(packed)]
#[derive(Default)]
//#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Tile {
    player: Pubkey,      // 玩家公钥 (32字节)
    state: u8,           // 格子状态 (1字节)
    health: u64,         // 生命值 (8字节)
    damage: u64,         // 伤害值 (8字节)
    range: u16,          // 攻击范围 (2字节)
    collect_reward: u64, // 收集奖励 (8字节)
    avatar: Pubkey,      // 头像公钥,用于客户端显示 (32字节)
    look_direction: u8,  // 朝向(上、右、下、左) (1字节)
    ship_level: u16,     // 船只等级 (2字节)
    start_health: u64,   // 初始生命值,用于客户端显示血条 (8字节)
}

// 游戏动作历史记录账户
#[account]
#[derive(InitSpace)]
pub struct GameActionHistory {
    id_counter: u64, // ID计数器

    #[max_len(100)]
    game_actions: Vec<GameAction>, // 游戏动作记录数组
}

// 游戏动作结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct GameAction {
    action_id: u64,  // 动作ID (8字节)
    action_type: u8, // 动作类型 (1字节)
    player: Pubkey,  // 玩家公钥 (32字节)
    target: Pubkey,  // 目标公钥 (32字节)
    damage: u64,     // 伤害值 (8字节)
}

impl GameDataAccount {
    // 打印游戏棋盘状态(仅用于本地调试)
    pub fn print(&mut self) {
        // print will only work locally for debugging otherwise it eats too much compute
        /*
        for x in 0..BOARD_SIZE_X {
            for y in 0..BOARD_SIZE_Y {
                let tile = self.board[x][y  ];
                if tile.state == STATE_EMPTY {
                    msg!("empty")
                } else {
                    msg!("{} {}", tile.player, tile.state)
                }
            }
        }*/
    }

    // 重置整个游戏棋盘
    pub fn reset(&mut self) -> Result<()> {
        for x in 0..BOARD_SIZE_X {
            for y in 0..BOARD_SIZE_Y {
                self.board[x][y].state = STATE_EMPTY
            }
        }
        Ok(())
    }

    // 重置指定玩家的船只
    pub fn reset_ship(&mut self, ship_owner: Pubkey) -> Result<()> {
        for x in 0..BOARD_SIZE_X {
            for y in 0..BOARD_SIZE_Y {
                if self.board[x][y].player == ship_owner && self.board[x][y].state == STATE_PLAYER {
                    self.board[x][y].state = STATE_EMPTY
                }
            }
        }
        Ok(())
    }

    // 计算两点间欧几里得距离
    pub fn euclidean_distance(x1: &usize, x2: &usize, y1: &usize, y2: &usize) -> f64 {
        let dx = (x1 - x2) as f64;
        let dy = (y1 - y2) as f64;
        (dx * dx + dy * dy).sqrt()
    }

    // 克苏鲁攻击逻辑
    pub fn cthulhu<'info>(
        &mut self,
        _player: AccountInfo,
        game_actions: &mut GameActionHistory,
        _chest_vault: AccountInfo,
        _vault_token_account: AccountInfo<'info>,
        _player_token_account: AccountInfo<'info>,
        _token_account_owner_pda: AccountInfo<'info>,
        _token_program: AccountInfo<'info>,
        _token_owner_bump: u8,
    ) -> Result<()> {
        let mut smallest_distance: f64 = 100000.0;
        let mut attacked_player_position: Option<(usize, usize)> = None;

        let cthulhu_position: (usize, usize) = (0, 0);

        // 寻找最近的玩家
        for x in 0..BOARD_SIZE_X {
            for y in 0..BOARD_SIZE_Y {
                let tile = self.board[x][y];
                if tile.state == STATE_PLAYER {
                    let distance =
                        Self::euclidean_distance(&x, &cthulhu_position.0, &y, &cthulhu_position.0);
                    if distance < smallest_distance {
                        smallest_distance = distance;
                        attacked_player_position = Some((x, y));
                    }
                }
            }
        }

        // 对找到的最近玩家进行攻击
        match attacked_player_position {
            None => {
                return Err(SevenSeasError::CouldNotFindAShipToAttack.into());
            }
            Some(val) => {
                let tile = &mut self.board[val.0][val.1];

                let mut rng = XorShift64 { a: tile.health };

                // 计算克苏鲁伤害值
                let chtulu_damage: u64 = 10;
                let damage_variant = ((chtulu_damage as f64) * 0.3).ceil() as u64;
                let damage = chtulu_damage + ((rng.next() % damage_variant) + 1);
                let option = tile.health.checked_sub(damage);
                match option {
                    None => {
                        tile.health = 0;
                    }
                    Some(val) => {
                        tile.health = val;
                    }
                }

                // 如果生命值为0,移除玩家
                if tile.health == 0 {
                    tile.state = STATE_EMPTY;
                }

                // 记录攻击动作
                let item = GameAction {
                    action_id: self.action_id,
                    action_type: GAME_ACTION_SHIP_CTHULUH_ATTACKED_SHIP,
                    player: tile.player.key(),
                    target: tile.player.key(),
                    damage: damage,
                };
                self.add_new_game_action(game_actions, item);

                msg!(
                    "Attack closes enemy is at {} {} with damage {}",
                    val.0,
                    val.1,
                    damage
                );
            }
        }

        Ok(())
    }

    // 船只射击逻辑
    pub fn shoot<'info>(
        &mut self,
        player: AccountInfo,
        game_actions: &mut GameActionHistory,
        chest_vault: AccountInfo,
        vault_token_account: AccountInfo<'info>,
        player_token_account: AccountInfo<'info>,
        token_account_owner_pda: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        token_owner_bump: u8,
    ) -> Result<()> {
        let mut player_position: Option<(usize, usize)> = None;

        // 寻找射击玩家的位置
        for x in 0..BOARD_SIZE_X {
            for y in 0..BOARD_SIZE_Y {
                let tile = self.board[x][y];
                if tile.state == STATE_PLAYER {
                    if tile.player == *player.key {
                        player_position = Some((x, y));
                    }
                    msg!("{} {}", tile.player, tile.state);
                }
            }
        }

        // 如果找到玩家位置,执行射击
        match player_position {
            None => {
                return Err(SevenSeasError::TriedToShootWithPlayerThatWasNotOnTheBoard.into());
            }
            Some(val) => {
                msg!("Player position x:{} y:{}", val.0, val.1);
                let player_tile: Tile = self.board[val.0][val.1];
                let range_usize: usize = usize::from(player_tile.range);
                let damage = player_tile.damage + 2;

                // 根据射程范围进行射击
                for range in 1..range_usize + 1 {
                    // 向左射击
                    if player_tile.look_direction % 2 == 0 && val.0 >= range {
                        self.attack_tile(
                            (val.0 - range, val.1),
                            damage,
                            player.clone(),
                            chest_vault.clone(),
                            game_actions,
                            &vault_token_account,
                            &player_token_account,
                            &token_account_owner_pda,
                            &token_program,
                            token_owner_bump,
                        )?;
                    }

                    // 向右射击
                    if player_tile.look_direction % 2 == 0 && val.0 < BOARD_SIZE_X - range {
                        self.attack_tile(
                            (val.0 + range, val.1),
                            damage,
                            player.clone(),
                            chest_vault.clone(),
                            game_actions,
                            &vault_token_account,
                            &player_token_account,
                            &token_account_owner_pda,
                            &token_program,
                            token_owner_bump,
                        )?;
                    }

                    // 向下射击
                    if player_tile.look_direction % 2 == 1 && val.1 < BOARD_SIZE_Y - range {
                        self.attack_tile(
                            (val.0, val.1 + range),
                            damage,
                            player.clone(),
                            chest_vault.clone(),
                            game_actions,
                            &vault_token_account,
                            &player_token_account,
                            &token_account_owner_pda,
                            &token_program,
                            token_owner_bump,
                        )?;
                    }

                    // 向上射击
                    if player_tile.look_direction % 2 == 1 && val.1 >= range {
                        self.attack_tile(
                            (val.0, val.1 - range),
                            damage,
                            player.clone(),
                            chest_vault.clone(),
                            game_actions,
                            &vault_token_account,
                            &player_token_account,
                            &token_account_owner_pda,
                            &token_program,
                            token_owner_bump,
                        )?;
                    }
                }

                // 记录射击动作
                let item = GameAction {
                    action_id: self.action_id,
                    action_type: GAME_ACTION_SHIP_SHOT,
                    player: player.key(),
                    target: player.key(),
                    damage: damage,
                };
                self.add_new_game_action(game_actions, item);
            }
        }

        Ok(())
    }

    // 添加新的游戏动作到历史记录
    fn add_new_game_action(
        &mut self,
        game_actions: &mut GameActionHistory,
        game_action: GameAction,
    ) {
        {
            let option_add = self.action_id.checked_add(1);
            match option_add {
                Some(val) => {
                    self.action_id = val;
                }
                None => {
                    self.action_id = 0;
                }
            }
        }
        game_actions.game_actions.push(game_action);
        // 保持历史记录在合理范围内
        /*
                drain() 方法的特点：
        会移除指定范围内的所有元素
        被移除的元素会被丢弃（在这个场景中我们不需要保留这些元素）
        剩余的元素会自动向前移动填补空缺
        这是一个原地操作，不会创建新的 Vec
        这是一个常见的"滚动窗口"模式，用于限制历史记录的大小。
                 */
        if game_actions.game_actions.len() > 100 {
            game_actions.game_actions.drain(0..5);
        }
    }

    // 攻击指定格子
    fn attack_tile<'info>(
        &mut self,
        attacked_position: (usize, usize),
        damage: u64,
        attacker: AccountInfo,
        chest_vault: AccountInfo,
        game_actions: &mut GameActionHistory,
        vault_token_account: &AccountInfo<'info>,
        player_token_account: &AccountInfo<'info>,
        token_account_owner_pda: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        token_owner_bump: u8,
    ) -> Result<()> {
        let mut attacked_tile: Tile = self.board[attacked_position.0][attacked_position.1];
        msg!("Attack x:{} y:{}", attacked_position.0, attacked_position.1);

        // 准备代币转账指令
        let transfer_instruction = Transfer {
            from: vault_token_account.to_account_info(),
            to: player_token_account.to_account_info(),
            authority: token_account_owner_pda.to_account_info(),
        };

        let seeds = &[b"token_account_owner_pda".as_ref(), &[token_owner_bump]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            token_program.to_account_info(),
            transfer_instruction,
            signer,
        );

        // 如果攻击目标是玩家
        if attacked_tile.state == STATE_PLAYER {
            let match_option = attacked_tile.health.checked_sub(damage);
            match match_option {
                None => {
                    attacked_tile.health = 0;
                    self.on_ship_died(attacked_position, attacked_tile, chest_vault, &attacker)?;
                    // 转移摧毁船只的奖励代币
                    anchor_spl::token::transfer(
                        cpi_ctx,
                        (attacked_tile.ship_level as u64)
                            * DESTROY_SHIP_COIN_REWARD
                            * TOKEN_DECIMAL_MULTIPLIER,
                    )?;

                    // 记录收集金币动作
                    let new_game_action = GameAction {
                        action_id: self.action_id,
                        action_type: GAME_ACTION_SHIP_COINS_COLLECTED,
                        player: attacker.key(),
                        target: attacked_tile.player.key(),
                        damage: DESTROY_SHIP_COIN_REWARD,
                    };
                    self.add_new_game_action(game_actions, new_game_action);
                }
                Some(value) => {
                    msg!("New health {}", value);
                    attacked_tile.health = value;
                    if value == 0 {
                        self.on_ship_died(
                            attacked_position,
                            attacked_tile,
                            chest_vault,
                            &attacker,
                        )?;
                        // 转移摧毁船只的奖励代币
                        anchor_spl::token::transfer(
                            cpi_ctx,
                            (attacked_tile.ship_level as u64)
                                * DESTROY_SHIP_COIN_REWARD
                                * TOKEN_DECIMAL_MULTIPLIER,
                        )?;
                        let item = GameAction {
                            action_id: self.action_id,
                            action_type: GAME_ACTION_SHIP_COINS_COLLECTED,
                            player: attacker.key(),
                            target: attacked_tile.player.key(),
                            damage: DESTROY_SHIP_COIN_REWARD,
                        };
                        self.add_new_game_action(game_actions, item);
                    }
                }
            };
            // 记录受到伤害动作
            let item = GameAction {
                action_id: self.action_id,
                action_type: GAME_ACTION_SHIP_TAKEN_DAMAGE,
                player: attacker.key(),
                target: attacked_tile.player.key(),
                damage,
            };
            self.add_new_game_action(game_actions, item);
        }
        Ok(())
    }

    // 处理船只死亡
    fn on_ship_died(
        &mut self,
        attacked_position: (usize, usize),
        attacked_tile: Tile,
        chest_vault: AccountInfo,
        attacker: &AccountInfo,
    ) -> Result<()> {
        msg!(
            "Enemy killed x:{} y:{} pubkey: {}",
            attacked_position.0,
            attacked_position.1,
            attacked_tile.player
        );
        self.board[attacked_position.0][attacked_position.1].state = STATE_EMPTY;
        // 转移奖励金额
        **chest_vault.try_borrow_mut_lamports()? -= attacked_tile.collect_reward;
        **attacker.try_borrow_mut_lamports()? += attacked_tile.collect_reward;
        Ok(())
    }

    // 移动指定玩家的船只
    pub fn move_in_direction<'info>(
        &mut self,
        direction: u8,
        player: AccountInfo,
        chest_vault: AccountInfo,
        vault_token_account: AccountInfo<'info>,
        player_token_account: AccountInfo<'info>,
        token_account_owner_pda: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        token_owner_bump: u8,
        game_actions: &mut GameActionHistory,
    ) -> Result<()> {
        // 1. 找到玩家当前位置
        let current_pos = self.find_player_position(player.key)?;

        // 2. 计算新位置
        let new_pos = self.calculate_new_position(current_pos, direction)?;

        // 3. 处理移动逻辑
        self.handle_movement(
            current_pos,
            new_pos,
            direction,
            player,
            chest_vault,
            vault_token_account,
            player_token_account,
            token_account_owner_pda,
            token_program,
            token_owner_bump,
            game_actions,
        )
    }

    // 查找玩家位置
    fn find_player_position(&self, player_key: &Pubkey) -> Result<(usize, usize)> {
        for x in 0..BOARD_SIZE_X {
            for y in 0..BOARD_SIZE_Y {
                let tile = &self.board[x][y];
                if tile.state == STATE_PLAYER && tile.player == *player_key {
                    return Ok((x, y));
                }
            }
        }
        Err(SevenSeasError::TriedToMovePlayerThatWasNotOnTheBoard.into())
    }

    // 计算新位置
    fn calculate_new_position(
        &self,
        current: (usize, usize),
        direction: u8,
    ) -> Result<(usize, usize)> {
        let (x, y) = current;
        match direction {
            0 if y > 0 => Ok((x, y - 1)),
            1 if x < BOARD_SIZE_X - 1 => Ok((x + 1, y)),
            2 if y < BOARD_SIZE_Y - 1 => Ok((x, y + 1)),
            3 if x > 0 => Ok((x - 1, y)),
            _ => Err(SevenSeasError::WrongDirectionInput.into()),
        }
    }

    // 处理移动逻辑
    fn handle_movement<'info>(
        &mut self,
        current_pos: (usize, usize),
        new_pos: (usize, usize),
        direction: u8,
        player: AccountInfo,
        chest_vault: AccountInfo,
        vault_token_account: AccountInfo<'info>,
        player_token_account: AccountInfo<'info>,
        token_account_owner_pda: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        token_owner_bump: u8,
        game_actions: &mut GameActionHistory,
    ) -> Result<()> {
        let new_tile = &self.board[new_pos.0][new_pos.1];

        match new_tile.state {
            STATE_EMPTY => {
                // 移动到空格子
                self.move_to_empty_tile(current_pos, new_pos, direction)
            }
            STATE_CHEST => {
                // 收集宝箱
                self.collect_chest(
                    current_pos,
                    new_pos,
                    direction,
                    player,
                    chest_vault,
                    vault_token_account,
                    player_token_account,
                    token_account_owner_pda,
                    token_program,
                    token_owner_bump,
                    game_actions,
                )
            }

            STATE_PLAYER => {
                // 攻击其他玩家
                self.attack_tile(
                    new_pos,
                    1,
                    player,
                    chest_vault,
                    game_actions,
                    &vault_token_account,
                    &player_token_account,
                    &token_account_owner_pda,
                    &token_program,
                    token_owner_bump,
                )
            }
            _ => Err(SevenSeasError::InvalidTileState.into()),
        }
    }

    // 移动到空格子
    fn move_to_empty_tile(
        &mut self,
        current_pos: (usize, usize),
        new_pos: (usize, usize),
        direction: u8,
    ) -> Result<()> {
        // 移动玩家
        self.board[new_pos.0][new_pos.1] = self.board[current_pos.0][current_pos.1];
        self.board[current_pos.0][current_pos.1].state = STATE_EMPTY;
        self.board[new_pos.0][new_pos.1].look_direction = direction;
        msg!("Moved player to new tile");
        Ok(())
    }

    // 收集宝箱
    fn collect_chest<'info>(
        &mut self,
        current_pos: (usize, usize),
        new_pos: (usize, usize),
        direction: u8,
        player: AccountInfo,
        chest_vault: AccountInfo,
        vault_token_account: AccountInfo<'info>,
        player_token_account: AccountInfo<'info>,
        token_account_owner_pda: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        token_owner_bump: u8,
        game_actions: &mut GameActionHistory,
    ) -> Result<()> {
        let chest_reward = self.board[new_pos.0][new_pos.1].collect_reward;

        // 移动玩家
        self.board[new_pos.0][new_pos.1] = self.board[current_pos.0][current_pos.1];
        self.board[current_pos.0][current_pos.1].state = STATE_EMPTY;
        self.board[new_pos.0][new_pos.1].look_direction = direction;

        // 转移SOL奖励
        **chest_vault.try_borrow_mut_lamports()? -= chest_reward;
        **player.try_borrow_mut_lamports()? += chest_reward;

        // 转移代币奖励
        let transfer_instruction = Transfer {
            from: vault_token_account,
            to: player_token_account,
            authority: token_account_owner_pda,
        };

        let seeds = &[b"token_account_owner_pda".as_ref(), &[token_owner_bump]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(token_program, transfer_instruction, signer);

        anchor_spl::token::transfer(cpi_ctx, CHEST_COIN_REWARD * TOKEN_DECIMAL_MULTIPLIER)?;

        // 记录收集金币动作
        let action = GameAction {
            action_id: self.action_id,
            action_type: GAME_ACTION_SHIP_COINS_COLLECTED,
            player: player.key(),
            target: player.key(),
            damage: CHEST_COIN_REWARD,
        };
        self.add_new_game_action(game_actions, action);

        msg!("Collected Chest");
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        for x in 0..BOARD_SIZE_X {
            for y in 0..BOARD_SIZE_Y {
                self.board[x][y].state = STATE_EMPTY;
            }
        }
        Ok(())
    }

    /// 生成玩家到游戏棋盘上
    ///
    /// # 参数
    /// * `player` - 玩家账户信息
    /// * `avatar` - 玩家头像公钥
    /// * `ship` - 玩家船只信息
    /// * `extra_health` - 额外生命值
    ///
    /// # 返回值
    /// * `Result<()>` - 成功返回Ok(()),失败返回错误
    ///
    /// # 功能说明
    /// 1. 遍历棋盘找出所有空格子
    /// 2. 检查玩家是否已存在
    /// 3. 随机选择一个空格子生成玩家
    /// 4. 根据船只等级设置攻击范围
    pub fn spawn_player(
        &mut self,
        player: AccountInfo,
        avatar: Pubkey,
        ship: &mut Ship,
        extra_health: u64,
    ) -> Result<()> {
        // 存储所有空格子的坐标
        let mut empty_slots: Vec<(usize, usize)> = Vec::new();

        // 遍历棋盘找出所有空格子
        for x in 0..BOARD_SIZE_X {
            for y in 0..BOARD_SIZE_Y {
                let tile = self.board[x][y];
                if tile.state == STATE_EMPTY {
                    empty_slots.push((x, y));
                } else if tile.player == player.key.clone() && tile.state == STATE_PLAYER {
                    // 如果玩家已存在则返回错误
                    return Err(SevenSeasError::PlayerAlreadyExists.into());
                }
            }
        }

        // 如果没有空格子则返回错误
        if empty_slots.is_empty() {
            return Err(SevenSeasError::BoardIsFull.into());
        }

        // 创建随机数生成器
        let mut rng = XorShift64 {
            a: empty_slots.len() as u64,
        };

        // 随机选择一个空格子
        let random_empty_slot = empty_slots[(rng.next() % (empty_slots.len() as u64)) as usize];
        msg!(
            "Player spawn at {} {}",
            random_empty_slot.0,
            random_empty_slot.1
        );

        // 根据船只等级设置攻击范围
        let range = match ship.upgrades {
            0 | 1 | 2 => {
                1 // 0-2级船只攻击范围为1
            }
            _ => {
                2 // 3级及以上船只攻击范围为2
            }
        };

        // 在选中的格子生成玩家
        self.board[random_empty_slot.0][random_empty_slot.1] = Tile {
            player: player.key.clone(),
            avatar: avatar.clone(),
            state: STATE_PLAYER,
            health: ship.health + extra_health,
            start_health: ship.health + extra_health,
            damage: ship.cannons,
            range,
            collect_reward: PLAYER_KILL_REWARD,
            look_direction: 0,
            ship_level: ship.upgrades,
        };

        Ok(())
    }

    /// 在游戏棋盘上生成宝箱
    ///
    /// # 参数
    /// * `player` - 玩家账户信息
    ///
    /// # 返回值
    /// * `Result<()>` - 成功返回Ok(()),失败返回错误
    ///
    /// # 功能说明
    /// 1. 遍历棋盘找出所有空格子
    /// 2. 随机选择一个空格子生成宝箱
    pub fn spawn_chest(&mut self, player: AccountInfo) -> Result<()> {
        // 存储所有空格子的坐标
        let mut empty_slots = Vec::new();

        // 遍历棋盘找出所有空格子
        for x in 0..BOARD_SIZE_X {
            for y in 0..BOARD_SIZE_Y {
                let tile = self.board[x][y];
                if tile.state == STATE_EMPTY {
                    empty_slots.push((x, y));
                } else {
                    //msg!("{}", tile.player);
                }
            }
        }

        // 如果没有空格子则返回错误
        if empty_slots.len() == 0 {
            return Err(SevenSeasError::BoardIsFull.into());
        }

        // 创建随机数生成器
        let mut rng = XorShift64 {
            a: (empty_slots.len() + 1) as u64,
        };

        // 随机选择一个空格子
        let random_empty_slot = empty_slots[(rng.next() % (empty_slots.len() as u64)) as usize];
        msg!(
            "Chest spawn at {} {}",
            random_empty_slot.0,
            random_empty_slot.1
        );

        // 在选中的格子生成宝箱
        self.board[random_empty_slot.0][random_empty_slot.1] = Tile {
            player: player.key.clone(),
            avatar: player.key.clone(),
            state: STATE_CHEST,
            health: 1,
            start_health: 1,
            damage: 0,
            range: 0,
            collect_reward: CHEST_REWARD,
            look_direction: 0,
            ship_level: 0,
        };

        Ok(())
    }
}

/// 宝箱金库账户结构体
#[account]
pub struct ChestVaultAccount {}

/// 简单的伪随机数生成器
pub struct XorShift64 {
    a: u64,
}

impl XorShift64 {
    /// 生成下一个随机数
    pub fn next(&mut self) -> u64 {
        let mut x = self.a;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.a = x;
        x
    }
}
