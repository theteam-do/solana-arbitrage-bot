// Solana相关依赖
use anchor_client::solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// 定义全局静态常量
lazy_static! {
    /// Solana代币程序ID
    pub static ref TOKEN_PROGRAM_ID: Pubkey = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    
    /// 关联代币账户程序ID
    pub static ref ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();
    
    /// Orca DEX程序ID
    pub static ref ORCA_PROGRAM_ID: Pubkey = Pubkey::from_str("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP").unwrap();
    
    /// Mercurial Finance程序ID
    pub static ref MERCURIAL_PROGRAM_ID: Pubkey = Pubkey::from_str("MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky").unwrap();
    
    /// 套利程序ID
    pub static ref ARB_PROGRAM_ID: Pubkey = Pubkey::from_str("CRQXfRGq3wTkjt7JkqhojPLiKLYLjHPGLebnfiiQB46T").unwrap();
    
    /// Saber程序ID
    pub static ref SABER_PROGRAM_ID : Pubkey = Pubkey::from_str("SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ").unwrap();
    
    /// Aldrin V1程序ID
    pub static ref ALDRIN_V1_PROGRAM_ID : Pubkey = Pubkey::from_str("AMM55ShdkoGRB5jVYPjWziwk8m5MpwyDgsMWHaMSQWH6").unwrap();
    
    /// Aldrin V2程序ID
    pub static ref ALDRIN_V2_PROGRAM_ID : Pubkey = Pubkey::from_str("CURVGoZn8zycx6FXwwevgBTB2gVvdbGTEpvMJDbgs2t4").unwrap();
    
    /// Serum DEX程序ID
    pub static ref SERUM_PROGRAM_ID : Pubkey = Pubkey::from_str("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin").unwrap();
}