use anchor_lang::prelude::*;

/// 船只结构体,用于存储船只的各项属性
#[account]
pub struct Ship {
    /// 当前生命值
    pub health: u64,
    /// 击杀数量
    pub kills: u16,
    /// 大炮数量
    pub cannons: u64,
    /// 升级次数
    pub upgrades: u16,
    /// 经验值
    pub xp: u16,
    /// 等级
    pub level: u16,
    /// 初始生命值
    pub start_health: u64,
}
