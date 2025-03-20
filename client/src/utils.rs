use crate::constants::*;
use crate::pool::PoolOperations;
use anchor_client::solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;
use std::str::FromStr;

/// 读取指定目录下的所有JSON文件路径
/// 
/// # 参数
/// * `dir` - 目录路径
/// 
/// # 返回值
/// 返回目录下所有JSON文件的路径列表
pub fn read_json_dir(dir: &String) -> Vec<String> {
    let _paths = fs::read_dir(dir).unwrap();
    let mut paths = Vec::new();
    for path in _paths {
        let p = path.unwrap().path();
        let path_str = p;
        match path_str.extension() {
            Some(ex) => {
                if ex == "json" {
                    let path = path_str.to_str().unwrap().to_string();
                    paths.push(path);
                }
            }
            None => {}
        }
    }
    paths
}

/// 将字符串转换为Solana公钥
/// 
/// # 参数
/// * `s` - 公钥字符串
/// 
/// # 返回值
/// 返回对应的Pubkey
pub fn str2pubkey(s: &str) -> Pubkey {
    Pubkey::from_str(s).unwrap()
}

/// 派生关联代币账户地址
/// 
/// # 参数
/// * `owner` - 所有者的公钥
/// * `mint` - 代币铸币地址
/// 
/// # 返回值
/// 返回派生的关联代币账户地址
pub fn derive_token_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(
        &[
            &owner.to_bytes(),
            &TOKEN_PROGRAM_ID.to_bytes(),
            &mint.to_bytes(),
        ],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    pda
}

/// 池报价包装器，包含池操作的引用计数指针
#[derive(Debug, Clone)]
pub struct PoolQuote(pub Rc<Box<dyn PoolOperations>>);

impl PoolQuote {
    /// 创建新的池报价实例
    pub fn new(quote: Rc<Box<dyn PoolOperations>>) -> Self {
        Self(quote)
    }
}

/// 池图结构，用于存储所有池之间的连接关系
/// 使用HashMap存储从源代币索引到边的映射
#[derive(Debug)]
pub struct PoolGraph(pub HashMap<PoolIndex, PoolEdge>);

/// 池索引，用于在图中唯一标识一个代币
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct PoolIndex(pub usize);

/// 池边结构，存储从一个代币到其他代币的所有可用池
#[derive(Debug, Clone)]
pub struct PoolEdge(pub HashMap<PoolIndex, Vec<PoolQuote>>);

impl PoolGraph {
    /// 创建新的空池图
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}
