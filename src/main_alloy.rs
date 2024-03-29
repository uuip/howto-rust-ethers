#![allow(unused_variables, dead_code)]

use chrono::Local;
use env_logger::fmt::Color;
use log::{debug, info, warn, LevelFilter, error};
use once_cell::sync::{Lazy, OnceCell};

use crate::Erc20Token::Erc20TokenEvents::TokenTransfer;
use crate::Erc20Token::{Erc20TokenCalls, Erc20TokenEvents};
use alloy_primitives::{Address, Bytes, TxHash, U256, U64};
use alloy_providers::provider::{Provider, TempProvider};
use alloy_rpc_client::RpcClient;
use alloy_rpc_types::{BlockId, CallInput, CallRequest, LegacyTransactionRequest};
use alloy_sol_types::{sol, SolCall, SolInterface};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::sync::Arc;
use alloy_rpc_types::BlockNumberOrTag::Latest;
use url::Url;
// use crate::erc20::*;
use crate::setting::Setting;

// mod erc20;
mod setting;

static CHAIN_ID: OnceCell<U64> = OnceCell::new();
static SETTING: Lazy<Setting, fn() -> Setting> = Lazy::new(Setting::init);

// const ABI_PATH: &str = "erc20_abi.json";
sol!(Erc20Token, "erc20_abi.json");

#[derive(Debug)]
struct Input {
    to: Address,
    value: U256,
    data: serde_json::Value,
}

// impl Detokenize for Input {
//     fn from_tokens(tokens: Vec<Token>) -> Result<Self, InvalidOutputType>
//     where
//         Self: Sized,
//     {
//         let e = || InvalidOutputType("data error".to_string());
//         if let [to, value, data] = &tokens[..] {
//             let to = to.clone().into_address().ok_or_else(e)?;
//             let value = value.clone().into_uint().ok_or_else(e)?;
//             let somedata = data.clone().into_string().ok_or_else(e)?;
//             let c = serde_json::from_str(&somedata).map_err(|_| e())?;
//             Ok(Self { to, value, data: c })
//         } else {
//             Err(e())
//         }
//     }
// }

// pub struct FixedH256(pub H256);
//
// impl Display for FixedH256 {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "0x")?;
//         for i in &self.0 .0 {
//             write!(f, "{:02x}", i)?;
//         }
//         Ok(())
//     }
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .format(|buf, record| {
            let mut level_style = buf.style();
            if record.level() == LevelFilter::Warn {
                level_style.set_color(Color::Ansi256(206_u8));
            }
            writeln!(
                buf,
                "[{} | line:{:<4}|{}]: {}",
                Local::now().format("%H:%M:%S"),
                record.line().unwrap_or(0),
                level_style.value(record.level()),
                level_style.value(record.args())
            )
        })
        .init();

    let url = Url::parse(&SETTING.rpc)?;
    let client = RpcClient::builder().reqwest_http(url);
    let w3 = Arc::new(Provider::new_with_client(client));
    let chain_id = w3.get_chain_id().await?;
    info!("{chain_id}");
    let c_addr=(&SETTING.token).parse::<Address>()?;

    // let c =
    //     Erc20Token::new(SETTING.token.parse::<Address>()?, w3.clone());
    let transaction_hash: TxHash =
        "0x079b409e03acb6b2f9985032de5325aaef242336c3eef2f007c32102a23fb66c".parse()?;
    //
    warn!("{}", format!("{:?}", transaction_hash));
    // warn!("{}", FixedH256(transaction_hash));
    //
    // 设置全局变量
    let chain_id = w3.get_chain_id().await?;
    CHAIN_ID
        .set(chain_id)
        .unwrap_or_else(|_| panic!("can't set chain_id"));
    let code=w3.get_code_at("0x37b856ef04754283D0712712F045257B3d55F9eB".parse()?, Latest.into()).await?;
    info!("{}",code);
    // info!("{:*^80}", "get chains's txpool");
    // let txpool = async {
    //     let txpool = w3.txpool_content().await?;
    //     for (addr, txs) in txpool.pending {
    //         info!("txpool {}", format!("{:?}", addr));
    //         for (nonce, tx) in txs {}
    //     }
    //     Ok::<(), anyhow::Error>(())
    // };
    // let _ = txpool.await;

    info!("{:*^80}", "decode_input");
    let decode_data = async {
        let tx = w3.get_transaction_by_hash(transaction_hash).await?;
        debug!("tx {:?}", &tx.input);
        let decode_input2 = Erc20TokenCalls::abi_decode(&tx.input, false)?;
        if let Erc20TokenCalls::tokenTransfer(v) = decode_input2 {
            info!("decode_data2 {:?}", v.message)
        };

        // let decode_input3 = c.decode_input::<TokenTransferCall, Bytes>(tx.input.clone())?;
        // info!("decode_data3 {:?}", decode_input3);
        // let decode_input4 = c.decode_input::<Input, Bytes>(tx.input)?;
        // info!("decode_data4 {:?}", decode_input4);
        Ok::<(), anyhow::Error>(())
    };
    let _ = decode_data.await;

    info!("{:*^80}", "call function");
    let call_function = async {
        let data: Bytes = Erc20Token::balanceOfCall::new((
            "0x44ea38b427fce87147dc034caae56f4a46cdfe98".parse::<Address>()?,
        ))
        .abi_encode()
        .into();
        let rst = w3
            .call(
                CallRequest {
                    to: Some(c_addr),
                    input:CallInput::new(data),
                    chain_id: Some(chain_id),
                    ..Default::default()
                },
                None,
            )
            .await?;
        let a=Erc20Token::balanceOfCall::abi_decode_returns(rst.as_ref(),false)?;
        info!("{:?}",a._0);
        info!("read_function {:?}", rst);
        Ok::<(), anyhow::Error>(())
    };
    let a = call_function.await;
    if let Err(e)=a{ error!("{e}")};


    // let event_topic2 = async {
    //     let event = c.event::<TokenTransferFilter>();
    //     let topic = event.filter.topics[0].clone().unwrap();
    //     if let Topic::Value(v) = topic {
    //         v
    //     } else {
    //         None
    //     }
    // };

    let event_topic = Erc20Token::TokenTransfer::SIGNATURE_HASH;
    Contract
    // let event_name = TokenTransferFilter::name();
    // let event_instance = c.token_transfer_filter();
    // let event_instance2 = c.event::<TokenTransferFilter>();
    // let event_instance3 = Contract::event_of_type::<TokenTransferFilter>(w3.clone());
    // let topic2 = event_topic2.await.unwrap();
    // assert_eq!(topic2, event_topic);
    //
    // info!("{:*^80}", "query_all_events");
    // async fn query_events(contract: &Erc20Token<Provider<Http>>) -> Result<(), anyhow::Error> {
    //     let start = 594933_u64;
    //     let events = contract.events().from_block(start).to_block(start + 2);
    //     let logs = events.query_with_meta().await?;
    //     for (decoded_log, meta) in logs {
    //         if let Erc20TokenEvents::TokenTransferFilter(f) = decoded_log {
    //             info!("{f:?}");
    //         }
    //     }
    //     Ok(())
    // }
    // let _ = query_events(&c).await;
    //
    // async fn query_specific_events(
    //     contract: &Erc20Token<Provider<Http>>,
    // ) -> Result<(), anyhow::Error> {
    //     let start = 594933_u64;
    //     let events = contract
    //         .event::<TokenTransferFilter>()
    //         .from_block(start)
    //         .to_block(start + 2);
    //     let logs: Vec<(TokenTransferFilter, LogMeta)> = events.query_with_meta().await?;
    //     for (decoded_log, meta) in logs {
    //         info!("{decoded_log:?}");
    //     }
    //     Ok(())
    // }
    // let _ = query_specific_events(&c).await;
    //
    // let query_events_from_rpc_api = async {
    //     let start = 594933_u64;
    //     let filter = Filter::new()
    //         .address(c.address())
    //         .from_block(start)
    //         .to_block(start + 2);
    //     let logs = w3.get_logs(&filter).await?;
    //     for log in logs {
    //         let decoded_log = Erc20TokenEvents::decode_log(&RawLog::from(log))?;
    //         if let Erc20TokenEvents::TokenTransferFilter(f) = decoded_log {
    //             info!("{f:?}");
    //         }
    //     }
    //     Ok::<(), anyhow::Error>(())
    // };
    //
    // info!(
    //     "{:*^80}",
    //     "process receipt logs with Erc20TokenEvents::decode_log"
    // );
    // let tx_receipt = w3.get_transaction_receipt(transaction_hash).await?.unwrap();
    // for log in tx_receipt.logs.iter() {
    //     let decoded_log = Erc20TokenEvents::decode_log(&RawLog::from(log.to_owned()))?;
    //     if let Erc20TokenEvents::TokenTransferFilter(f) = decoded_log {
    //         info!("{f:?}");
    //     }
    // }
    //
    // info!(
    //     "{:*^80}",
    //     "process receipt logs with parse_log::<TokenTransferFilter>"
    // );
    // for log in tx_receipt.logs.into_iter() {
    //     if event_topic == log.topics[0] {
    //         // let decoded_log =
    //         //     <TokenTransferFilter as EthLogDecode>::decode_log(&RawLog::from(log.to_owned()))?;
    //         let decoded_log: TokenTransferFilter = parse_log::<TokenTransferFilter>(log)?;
    //         info!("parse_log::<TokenTransferFilter> {:?}", decoded_log)
    //     }
    // }
    //
    // let _ = all_events_of_contract(&c).await;
    Ok(())
}

// fn construct_abi() -> Result<(), Box<dyn std::error::Error>> {
//     let abi_file = File::open(ABI_PATH)?;
//     let reader = BufReader::new(abi_file);
//     let abi = Abi::load(reader)?;
//     let all_events = &abi.events;
//     info!("{:?}", all_events);
//     Ok(())
// }
//
// async fn all_events_of_contract(c: &Erc20Token<Provider<Http>>) {
//     for x in &mut c.abi().events() {
//         info!("{:?} {:?}", x.name, x.signature());
//     }
// }

pub trait AnyExt {
    fn type_name(&self) -> &'static str;
}

impl<T> AnyExt for T {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}
