// Solana相关依赖
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};

use anchor_client::{Client, Cluster};

// 标准库依赖
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;
use std::str::FromStr;
use std::borrow::Borrow;
use std::vec;

// 命令行参数解析
use clap::Parser;

// 日志相关
use log::{debug, info, warn};
use solana_sdk::account::Account;

// 本地模块导入
use client::arb::*;           // 套利相关功能
use client::constants::*;      // 常量定义
use client::pool::{pool_factory, PoolDir, PoolOperations, PoolType};  // 流动性池相关
use client::serialize::token::unpack_token_account;  // Token账户序列化
use client::utils::{
    derive_token_address,     // Token地址派生
    read_json_dir,           // JSON文件读取
    PoolEdge,               // 池边缘定义
    PoolGraph,              // 池图结构
    PoolIndex,              // 池索引
    PoolQuote,             // 池报价
};

/// 命令行参数结构
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// 指定运行的集群类型 (localnet/mainnet)
    #[clap(short, long)]
    pub cluster: String,
}

/// 将池添加到图中的辅助函数
/// 
/// # 参数
/// * `graph` - 要修改的池图
/// * `idx0` - 第一个代币的索引
/// * `idx1` - 第二个代币的索引
/// * `quote` - 池的报价信息
fn add_pool_to_graph<'a>(
    graph: &mut PoolGraph,
    idx0: PoolIndex,
    idx1: PoolIndex,
    quote: &PoolQuote,
) {
    // idx0 = A, idx1 = B
    let edges = graph
        .0
        .entry(idx0)
        .or_insert_with(|| PoolEdge(HashMap::new()));
    let quotes = edges.0.entry(idx1).or_insert_with(|| vec![]);
    quotes.push(quote.clone());
}

/// 主函数 - 套利机器人的入口点
fn main() {
    // 解析命令行参数
    let args = Args::parse();
    let cluster = match args.cluster.as_str() {
        "localnet" => Cluster::Localnet,
        "mainnet" => Cluster::Mainnet,
        _ => panic!("invalid cluster type"),
    };

    // 初始化日志系统
    env_logger::init();

    // 根据不同环境选择密钥对路径
    let owner_kp_path = match cluster {
        Cluster::Localnet => "../../mainnet_fork/localnet_owner.key",
        Cluster::Mainnet => {
            "/Users/edgar/.config/solana/uwuU3qc2RwN6CpzfBAhg6wAxiEx138jy5wB3Xvx18Rw.json"
        }
        _ => panic!("shouldnt get here"),
    };

    // 设置RPC连接
    // 对于主网使用Jito RPC，其他情况使用默认RPC
    let connection_url = match cluster {
        Cluster::Mainnet => {
            "https://mainnet.rpc.jito.wtf/?access-token=746bee55-1b6f-4130-8347-5e1ea373333f"
        }
        _ => cluster.url(),
    };
    info!("using connection: {}", connection_url);

    // 创建RPC客户端连接
    let connection = RpcClient::new_with_commitment(connection_url, CommitmentConfig::confirmed());
    let send_tx_connection =
        RpcClient::new_with_commitment(cluster.url(), CommitmentConfig::confirmed());

    // 设置Anchor客户端
    let owner = read_keypair_file(owner_kp_path.clone()).unwrap();
    let rc_owner = Rc::new(owner);
    let provider = Client::new_with_options(
        cluster.clone(),
        rc_owner.clone(),
        CommitmentConfig::confirmed(),
    );
    let program = provider.program(*ARB_PROGRAM_ID);

    // 定义流动性池的JSON配置目录
    let mut pool_dirs = vec![];

    // 添加Orca协议的池配置
    let orca_dir = PoolDir {
        tipe: PoolType::OrcaPoolType,
        dir_path: "../pools/orca".to_string(),
    };
    pool_dirs.push(orca_dir);

    // 添加Mercurial协议的池配置
    let mercurial_dir = PoolDir {
        tipe: PoolType::MercurialPoolType,
        dir_path: "../pools/mercurial".to_string(),
    };
    pool_dirs.push(mercurial_dir);

    // 添加Saber协议的池配置
    let saber_dir = PoolDir {
        tipe: PoolType::SaberPoolType,
        dir_path: "../pools/saber/".to_string(),
    };
    pool_dirs.push(saber_dir);

    // 初始化数据结构
    let mut token_mints = vec![];           // 所有代币的铸币地址
    let mut pools = vec![];                 // 所有流动性池
    let mut update_pks = vec![];           // 需要更新的公钥列表
    let mut update_pks_lengths = vec![];    // 每个池需要更新的公钥数量
    let mut all_mint_idxs = vec![];        // 所有铸币索引
    let mut mint2idx = HashMap::new();      // 铸币地址到索引的映射
    let mut graph_edges = vec![];          // 图的边缘集合

    info!("extracting pool + mints...");
    // 遍历所有池目录，处理每个池的配置
    for pool_dir in pool_dirs {
        debug!("pool dir: {:#?}", pool_dir);
        let pool_paths = read_json_dir(&pool_dir.dir_path);

        for pool_path in pool_paths {
            // 读取并解析池配置
            let json_str = std::fs::read_to_string(&pool_path).unwrap();
            let pool = pool_factory(&pool_dir.tipe, &json_str);

            // 获取池中的代币铸币地址
            let pool_mints = pool.get_mints();
            if pool_mints.len() != 2 {
                // 仅支持双代币池
                warn!("skipping pool with mints != 2: {:?}", pool_path);
                continue;
            }

            // 记录池信息用于构建图
            let mut mint_idxs = vec![];
            for mint in pool_mints {
                let idx;
                if !token_mints.contains(&mint) {
                    // 新的铸币地址
                    idx = token_mints.len();
                    mint2idx.insert(mint, idx);
                    token_mints.push(mint);
                    graph_edges.push(HashSet::new());
                } else {
                    // 已存在的铸币地址
                    idx = *mint2idx.get(&mint).unwrap();
                }
                mint_idxs.push(idx);
            }

            // 获取需要更新的账户信息
            let update_accounts = pool.get_update_accounts();
            update_pks_lengths.push(update_accounts.len());
            update_pks.push(update_accounts);

            let mint0_idx = mint_idxs[0];
            let mint1_idx = mint_idxs[1];

            all_mint_idxs.push(mint0_idx);
            all_mint_idxs.push(mint1_idx);

            // 记录图的边，确保双向连接
            if !graph_edges[mint0_idx].contains(&mint1_idx) {
                graph_edges[mint0_idx].insert(mint1_idx);
            }
            if !graph_edges[mint1_idx].contains(&mint0_idx) {
                graph_edges[mint1_idx].insert(mint0_idx);
            }

            pools.push(pool);
        }
    }
    let mut update_pks = update_pks.concat();

    info!("added {:?} mints", token_mints.len());
    info!("added {:?} pools", pools.len());

    // 设置USDC作为起始代币
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let start_mint = usdc_mint;
    let start_mint_idx = *mint2idx.get(&start_mint).unwrap();

    let owner: &Keypair = rc_owner.borrow();
    let owner_start_addr = derive_token_address(&owner.pubkey(), &start_mint);

    // 添加所有者的起始代币账户到更新列表
    update_pks.push(owner_start_addr);

    // 获取所有池的当前状态
    info!("getting pool amounts...");
    let mut update_accounts = vec![];
    // 由于RPC限制，每次最多获取99个账户
    for token_addr_chunk in update_pks.chunks(99) {
        let accounts = connection.get_multiple_accounts(token_addr_chunk).unwrap();
        update_accounts.push(accounts);
    }
    let mut update_accounts = update_accounts
        .concat()
        .into_iter()
        .filter(|s| s.is_some())
        .collect::<Vec<Option<Account>>>();

    info!("update accounts is {:?}", update_accounts.len());
    println!("accounts: {:#?}", update_accounts.clone());
    
    // 获取初始代币账户信息
    let init_token_acc = update_accounts.pop().unwrap().unwrap();
    let init_token_balance = unpack_token_account(&init_token_acc.data).amount as u128;
    info!(
        "init token acc: {:?}, balance: {:#}",
        init_token_acc, init_token_balance
    );
    info!("starting balance = {}", init_token_balance);

    // 构建交易图
    info!("setting up exchange graph...");
    let mut graph = PoolGraph::new();
    let mut pool_count = 0;
    let mut account_ptr = 0;

    // 将所有池添加到图中
    for pool in pools.into_iter() {
        let length = update_pks_lengths[pool_count];
        let _account_slice = &update_accounts[account_ptr..account_ptr + length].to_vec();
        account_ptr += length;

        // 添加池到图中（双向）
        let idxs = &all_mint_idxs[pool_count * 2..(pool_count + 1) * 2].to_vec();
        let idx0 = PoolIndex(idxs[0]);
        let idx1 = PoolIndex(idxs[1]);

        let mut pool_ptr = PoolQuote::new(Rc::new(pool));
        add_pool_to_graph(&mut graph, idx0, idx1, &mut pool_ptr.clone());
        add_pool_to_graph(&mut graph, idx1, idx0, &mut pool_ptr);

        pool_count += 1;
    }

    // 创建套利器实例
    let arbitrager = Arbitrager {
        token_mints,
        graph_edges,
        graph,
        cluster,
        owner: rc_owner,
        program,
        connection: send_tx_connection,
    };

    info!("searching for arbitrages...");
    let min_swap_amount = 10_u128.pow(6_u32); // scaled! -- 1 USDC
    let mut swap_start_amount = init_token_balance; // scaled!
    let mut sent_arbs = HashSet::new(); // track what arbs we did with a larger size

    for _ in 0..4 {
        arbitrager.brute_force_search(
            start_mint_idx,
            swap_start_amount,
            swap_start_amount,
            vec![start_mint_idx],
            vec![],
            &mut sent_arbs,
        );

        swap_start_amount /= 2; // half input amount and search again
        if swap_start_amount < min_swap_amount {
            break;
        } // dont get too small
    }
}
