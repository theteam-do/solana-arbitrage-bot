//! Solana跨池套利机器人客户端库
//! 
//! 该库实现了在Solana区块链上进行自动化套利交易的功能。
//! 它支持多个DEX协议，包括Orca、Mercurial、Saber、Aldrin和Serum。
//! 通过构建交易图和寻找套利机会来实现利润最大化。

// 套利核心逻辑模块
pub mod arb;

// 序列化相关模块
pub mod serialize;

// 通用工具函数模块
pub mod utils;

// 流动性池工具函数模块
pub mod pool_utils; 

// 错误处理模块
pub mod error; 

// 流动性池接口和实现模块
pub mod pool; 

// 常量定义模块
pub mod constants; 

// 测试模块
pub mod tests;

// 各DEX协议的池实现模块
pub mod pools; 

// 启用lazy_static宏，用于定义静态变量
#[macro_use]
extern crate lazy_static;
