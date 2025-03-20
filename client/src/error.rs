//! 错误类型定义模块

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// 代币交换程序可能返回的错误类型
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum SwapError {
    // 账户相关错误 (0-4)
    /// 账户已被使用，无法初始化
    #[error("Swap account already in use")]
    AlreadyInUse,
    /// 程序地址与通过种子生成的值不匹配
    #[error("Invalid program address generated from bump seed and key")]
    InvalidProgramAddress,
    /// 输入账户的所有者不是程序生成的地址
    #[error("Input account owner is not the program address")]
    InvalidOwner,
    /// 池代币输出的所有者是程序生成的地址
    #[error("Output pool account owner cannot be the program address")]
    InvalidOutputOwner,
    /// 账户反序列化结果不是预期的Mint类型
    #[error("Deserialized account is not an SPL Token mint")]
    ExpectedMint,

    // 账户状态错误 (5-9)
    /// 账户反序列化结果不是预期的Account类型
    #[error("Deserialized account is not an SPL Token account")]
    ExpectedAccount,
    /// 输入代币账户余额为空
    #[error("Input token account empty")]
    EmptySupply,
    /// 池代币铸币账户的供应量不为零
    #[error("Pool token mint has a non-zero supply")]
    InvalidSupply,
    /// 代币账户有委托人
    #[error("Token account has a delegate")]
    InvalidDelegate,
    /// 输入代币无效
    #[error("InvalidInput")]
    InvalidInput,

    // 账户地址错误 (10-14)
    /// 提供的交换代币账户地址不正确
    #[error("Address of the provided swap token account is incorrect")]
    IncorrectSwapAccount,
    /// 提供的池代币铸币地址不正确
    #[error("Address of the provided pool token mint is incorrect")]
    IncorrectPoolMint,
    /// 输出代币无效
    #[error("InvalidOutput")]
    InvalidOutput,
    /// 由于溢出或下溢导致的计算失败
    #[error("General calculation failure due to overflow or underflow")]
    CalculationFailure,
    /// 传入的指令编号无效
    #[error("Invalid instruction")]
    InvalidInstruction,

    // 交易验证错误 (15-19)
    /// 交换输入代币账户使用了相同的铸币地址
    #[error("Swap input token accounts have the same mint")]
    RepeatedMint,
    /// 交换指令超出了期望的滑点限制
    #[error("Swap instruction exceeds desired slippage limit")]
    ExceededSlippage,
    /// 代币账户有关闭权限
    #[error("Token account has a close authority")]
    InvalidCloseAuthority,
    /// 池代币铸币账户有冻结权限
    #[error("Pool token mint has a freeze authority")]
    InvalidFreezeAuthority,
    /// 池费用代币账户不正确
    #[error("Pool fee token account incorrect")]
    IncorrectFeeAccount,

    // 计算和费用错误 (20-24)
    /// 给定的池代币数量导致交易代币数量为零
    #[error("Given pool token amount results in zero trading tokens")]
    ZeroTradingTokens,
    /// 由于溢出、下溢或意外的0导致费用计算失败
    #[error("Fee calculation failed due to overflow, underflow, or unexpected 0")]
    FeeCalculationFailure,
    /// 转换为u64类型时发生溢出或下溢
    #[error("Conversion to u64 failed with an overflow or underflow")]
    ConversionFailure,
    /// 提供的费用不符合程序所有者的约束条件
    #[error("The provided fee does not match the program owner's constraints")]
    InvalidFee,
    /// 提供的代币程序ID与交换预期的不匹配
    #[error("The provided token program does not match the token program expected by the swap")]
    IncorrectTokenProgramId,

    // 曲线相关错误 (25-27)
    /// 程序所有者不支持提供的曲线类型
    #[error("The provided curve type is not supported by the program owner")]
    UnsupportedCurveType,
    /// 提供的曲线参数无效
    #[error("The provided curve parameters are invalid")]
    InvalidCurve,
    /// 无法在给定曲线上执行操作
    #[error("The operation cannot be performed on the given curve")]
    UnsupportedCurveOperation,
}

/// 实现从SwapError到ProgramError的转换
impl From<SwapError> for ProgramError {
    fn from(e: SwapError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// 实现错误解码接口
impl<T> DecodeError<T> for SwapError {
    fn type_of() -> &'static str {
        "Swap Error"
    }
}



