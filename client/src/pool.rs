// Solana相关依赖
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::Program;
use solana_sdk::account::Account;
use solana_sdk::instruction::Instruction;

// 本地模块导入
use crate::pools::*;
use std::fmt::Debug;
use anchor_client::Cluster;

/// 流动性池目录结构，用于配置不同类型池的JSON文件位置
#[derive(Debug)]
pub struct PoolDir {
    pub tipe: PoolType,        // 池类型
    pub dir_path: String,      // 配置文件目录路径
}

/// 支持的流动性池类型枚举
#[derive(Debug)]
pub enum PoolType {
    OrcaPoolType,         // Orca DEX池
    MercurialPoolType,    // Mercurial Finance池
    SaberPoolType,        // Saber池
    AldrinPoolType,       // Aldrin池
    SerumPoolType,        // Serum DEX池
}

/// 流动性池工厂函数，根据类型和JSON配置创建相应的池实例
/// 
/// # 参数
/// * `tipe` - 池类型
/// * `json_str` - JSON格式的池配置
/// 
/// # 返回值
/// 返回实现了PoolOperations trait的池实例
pub fn pool_factory(tipe: &PoolType, json_str: &String) -> Box<dyn PoolOperations> {
    match tipe {
        PoolType::OrcaPoolType => {
            let pool: OrcaPool = serde_json::from_str(json_str).unwrap();
            Box::new(pool)
        }
        PoolType::MercurialPoolType => {
            let pool: MercurialPool = serde_json::from_str(json_str).unwrap();
            Box::new(pool)
        }
        PoolType::SaberPoolType => {
            let pool: SaberPool = serde_json::from_str(json_str).unwrap();
            Box::new(pool)
        }
        PoolType::AldrinPoolType => {
            let pool: AldrinPool = serde_json::from_str(json_str).unwrap();
            Box::new(pool)
        }
        PoolType::SerumPoolType => {
            let pool: SerumPool = serde_json::from_str(json_str).unwrap();
            Box::new(pool)
        }
    }
}

/// 流动性池操作trait，定义了所有池必须实现的接口
pub trait PoolOperations: Debug {
    /// 获取池的名称
    fn get_name(&self) -> String;
    
    /// 获取需要更新的账户公钥列表
    fn get_update_accounts(&self) -> Vec<Pubkey>;
    
    /// 设置账户信息
    /// 
    /// # 参数
    /// * `accounts` - 账户信息列表
    /// * `cluster` - 运行的集群类型
    fn set_update_accounts(&mut self, accounts: Vec<Option<Account>>, cluster: Cluster);

    /// 根据铸币地址获取代币账户地址
    fn mint_2_addr(&self, mint: &Pubkey) -> Pubkey;
    
    /// 获取池中所有代币的铸币地址
    fn get_mints(&self) -> Vec<Pubkey>;
    
    /// 获取代币的精度
    fn mint_2_scale(&self, mint: &Pubkey) -> u64;

    /// 计算交换后获得的代币数量（已考虑精度）
    /// 
    /// # 参数
    /// * `amount_in` - 输入代币数量
    /// * `mint_in` - 输入代币铸币地址
    /// * `mint_out` - 输出代币铸币地址
    /// 
    /// # 返回值
    /// 返回预期获得的输出代币数量
    fn get_quote_with_amounts_scaled(
        &self,
        amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
    ) -> u128;

    /// 生成交换指令
    /// 
    /// # 参数
    /// * `program` - Anchor程序实例
    /// * `owner` - 交易发起者的公钥
    /// * `mint_in` - 输入代币铸币地址
    /// * `mint_out` - 输出代币铸币地址
    /// 
    /// # 返回值
    /// 返回执行交换所需的指令列表
    fn swap_ix(
        &self,
        program: &Program,
        owner: &Pubkey,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
    ) -> Vec<Instruction>;

    /// 检查是否支持两个代币之间的交易（用于测试）
    fn can_trade(&self, mint_in: &Pubkey, mint_out: &Pubkey) -> bool;
}

// clone_trait_object!(PoolOperations);
