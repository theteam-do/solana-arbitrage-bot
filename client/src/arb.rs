use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;

use anchor_client::solana_sdk::pubkey::Pubkey;

use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::{Cluster, Program};
use std::collections::{HashMap, HashSet};

use solana_sdk::instruction::Instruction;
use solana_sdk::transaction::Transaction;

use std::borrow::Borrow;
use std::rc::Rc;

use std::vec;

use log::info;

use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;

use crate::pool::PoolOperations;

use crate::utils::{derive_token_address, PoolGraph, PoolIndex, PoolQuote};

/// 套利器结构体，包含执行套利所需的所有状态
pub struct Arbitrager {
    pub token_mints: Vec<Pubkey>,              // 所有代币的铸币地址
    pub graph_edges: Vec<HashSet<usize>>,      // 用于快速搜索图的边缘集合
    pub graph: PoolGraph,                      // 完整的池图结构
    pub cluster: Cluster,                      // 运行的集群类型(mainnet/localnet)
    pub owner: Rc<Keypair>,                    // 套利者的密钥对
    pub program: Program,                      // Anchor程序实例
    pub connection: RpcClient,                 // RPC客户端连接
}

impl Arbitrager {
    /// 暴力搜索套利机会
    /// 
    /// # 参数
    /// * `start_mint_idx` - 起始代币的索引
    /// * `init_balance` - 初始余额
    /// * `curr_balance` - 当前余额
    /// * `path` - 当前搜索路径
    /// * `pool_path` - 当前路径上的池
    /// * `sent_arbs` - 已发送的套利交易集合(用于去重)
    pub fn brute_force_search(
        &self,
        start_mint_idx: usize,
        init_balance: u128,
        curr_balance: u128,
        path: Vec<usize>,
        pool_path: Vec<PoolQuote>,
        sent_arbs: &mut HashSet<String>,
    ) {
        let src_curr = path[path.len() - 1];  // 获取路径中最后一个代币
        let src_mint = self.token_mints[src_curr];

        let out_edges = &self.graph_edges[src_curr];

        // 路径长度限制为4，因为Solana交易大小有限制
        // path = 4 表示 A -> B -> C -> D
        if path.len() == 4 {
            return;
        };

        // 遍历所有可能的目标代币
        for dst_mint_idx in out_edges {
            let pools = self
                .graph
                .0
                .get(&PoolIndex(src_curr))
                .unwrap()
                .0
                .get(&PoolIndex(*dst_mint_idx))
                .unwrap();

            // 避免循环，除非是回到起始代币
            if path.contains(dst_mint_idx) && *dst_mint_idx != start_mint_idx {
                continue;
            }

            let dst_mint_idx = *dst_mint_idx;
            let dst_mint = self.token_mints[dst_mint_idx];

            // 遍历所有可用的池
            for pool in pools {
                // 计算交换后的余额
                let new_balance =
                    pool.0
                        .get_quote_with_amounts_scaled(curr_balance, &src_mint, &dst_mint);

                let mut new_path = path.clone();
                new_path.push(dst_mint_idx);

                let mut new_pool_path = pool_path.clone();
                new_pool_path.push(pool.clone());

                if dst_mint_idx == start_mint_idx {
                    // 如果回到起始代币，检查是否有利可图
                    if new_balance > init_balance {
                        info!("found arbitrage: {:?} -> {:?}", init_balance, new_balance);

                        // 检查这个套利是否已经发送过
                        // 键格式 = {代币路径}{池名称}
                        let mint_keys: Vec<String> =
                            new_path.clone().iter_mut().map(|i| i.to_string()).collect();
                        let pool_keys: Vec<String> =
                            new_pool_path.iter().map(|p| p.0.get_name()).collect();
                        let arb_key = format!("{}{}", mint_keys.join(""), pool_keys.join(""));
                        if sent_arbs.contains(&arb_key) {
                            info!("arb already sent...");
                            continue;  // 避免重复发送相同的套利交易
                        } else {
                            sent_arbs.insert(arb_key);
                        }

                        // 构建并发送套利指令
                        let ixs = self.get_arbitrage_instructions(
                            init_balance,
                            &new_path,
                            &new_pool_path,
                        );
                        self.send_ixs(ixs);
                    }
                } else if !path.contains(&dst_mint_idx) {
                    // 继续深度搜索
                    self.brute_force_search(
                        start_mint_idx,
                        init_balance,
                        new_balance,
                        new_path,
                        new_pool_path,
                        sent_arbs,
                    );
                }
            }
        }
    }

    /// 构建套利交易的指令序列
    /// 
    /// # 参数
    /// * `swap_start_amount` - 起始交换金额
    /// * `mint_idxs` - 代币路径的索引
    /// * `pools` - 使用的池序列
    fn get_arbitrage_instructions(
        &self,
        swap_start_amount: u128,
        mint_idxs: &Vec<usize>,
        pools: &Vec<PoolQuote>,
    ) -> Vec<Instruction> {
        let mut ixs = vec![];
        
        // 获取交换状态PDA
        let (swap_state_pda, _) =
            Pubkey::find_program_address(&[b"swap_state"], &self.program.id());

        let src_mint = self.token_mints[mint_idxs[0]];
        let src_ata = derive_token_address(&self.owner.pubkey(), &src_mint);

        // 添加初始化交换指令
        let ix = self
            .program
            .request()
            .accounts(tmp_accounts::TokenAndSwapState {
                src: src_ata,
                swap_state: swap_state_pda,
            })
            .args(tmp_ix::StartSwap {
                swap_input: swap_start_amount as u64,
            })
            .instructions()
            .unwrap();
        ixs.push(ix);

        // 添加所有交换指令
        for i in 0..mint_idxs.len() - 1 {
            let [mint_idx0, mint_idx1] = [mint_idxs[i], mint_idxs[i + 1]];
            let [mint0, mint1] = [self.token_mints[mint_idx0], self.token_mints[mint_idx1]];
            let pool = &pools[i];

            let swap_ix = pool
                .0
                .swap_ix(&self.program, &self.owner.pubkey(), &mint0, &mint1);
            ixs.push(swap_ix);
        }

        // 添加利润检查或回滚指令
        let ix = self
            .program
            .request()
            .accounts(tmp_accounts::TokenAndSwapState {
                src: src_ata,
                swap_state: swap_state_pda,
            })
            .args(tmp_ix::ProfitOrRevert {})
            .instructions()
            .unwrap();
        ixs.push(ix);

        ixs.concat()
    }

    /// 发送交易指令
    /// 
    /// # 参数
    /// * `ixs` - 要发送的指令序列
    fn send_ixs(&self, ixs: Vec<Instruction>) {
        let owner: &Keypair = self.owner.borrow();
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&owner.pubkey()),
            &[owner],
            self.connection.get_latest_blockhash().unwrap(),
        );

        // 根据不同环境执行不同操作
        if self.cluster == Cluster::Localnet {
            // 本地网络：模拟交易
            let res = self.connection.simulate_transaction(&tx).unwrap();
            println!("{:#?}", res);
        } else if self.cluster == Cluster::Mainnet {
            // 主网：实际发送交易
            let signature = self
                .connection
                .send_transaction_with_config(
                    &tx,
                    RpcSendTransactionConfig {
                        skip_preflight: true,  // 跳过预检以加快速度
                        ..RpcSendTransactionConfig::default()
                    },
                )
                .unwrap();
            println!("signature: {:?}", signature);
        }
    }
}
