#![allow(unused_variables, dead_code)]

use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use ethers::abi::{Abi, Detokenize, InvalidOutputType, RawLog, Token};
use ethers::prelude::*;
use log::{debug, info, LevelFilter};
use once_cell::sync::{Lazy, OnceCell};
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

use crate::setting::Setting;
use erc20::*;

mod erc20;
mod setting;

static CHAIN_ID: OnceCell<U256> = OnceCell::new();
static SETTING: Lazy<Setting, fn() -> Setting> = Lazy::new(Setting::init);

const ABI_PATH: &str = "erc20_abi.json";
// abigen!(Erc20Token, "erc20_abi.json");

#[derive(Debug)]
struct Input {
    to: Address,
    value: U256,
    data: serde_json::Value,
}

impl Detokenize for Input {
    fn from_tokens(tokens: Vec<Token>) -> Result<Self, InvalidOutputType>
    where
        Self: Sized,
    {
        let e = InvalidOutputType("data error".to_string());
        if let [to, value, data] = &tokens[..] {
            let to = to.clone().into_address().ok_or(e.clone())?;
            let value = value.clone().into_uint().ok_or(e.clone())?;
            let somedata = data.clone().into_string().ok_or(e.clone())?;
            let c = serde_json::from_str(&somedata).map_err(|_| e.clone())?;
            Ok(Self { to, value, data: c })
        } else {
            Err(e)
        }
    }
}

pub struct FixedH256(pub H256);

impl Display for FixedH256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        for i in &self.0 .0 {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    TermLogger::init(
        LevelFilter::Debug,
        ConfigBuilder::new().build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;
    let _=rust_file_generation();

    let w3: Arc<Provider<Http>> = Arc::new(Provider::<Http>::try_from(&SETTING.rpc)?);
    let c = Erc20Token::new(SETTING.token.parse::<Address>()?, w3.clone());
    let transaction_hash: TxHash =
        "0x079b409e03acb6b2f9985032de5325aaef242336c3eef2f007c32102a23fb66c".parse()?;

    // 设置全局变量
    let chain_id = w3.get_chainid().await?;
    CHAIN_ID
        .set(chain_id)
        .unwrap_or_else(|_| panic!("can't set chain_id"));

    let txpool = async {
        let txpool = w3.txpool_content().await?;
        for (addr, txs) in txpool.pending {
            info!("txpool {}", format!("{:?}", addr));
            for (nonce, tx) in txs {}
        }
        Ok::<(), anyhow::Error>(())
    };

    let decode_data = async {
        let tx = w3.get_transaction(transaction_hash).await?.unwrap();
        let decode_input = c.decode_input::<Input, Bytes>(tx.input)?;
        info!("decode_data {:?}", decode_input);
        Ok::<(), anyhow::Error>(())
    };

    let read_function = async {
        let aa = c
            .balance_of("0x44ea38b427fce87147dc034caae56f4a46cdfe98".parse::<Address>()?)
            .call()
            .await?;
        info!("read_function {:?}", aa);
        Ok::<(), anyhow::Error>(())
    };

    let construct_topic = async {
        // let event = Contract::event_of_type::<TokenTransferFilter>(w3.clone());
        let event: Event<Arc<Provider<Http>>, Provider<Http>, TokenTransferFilter> =
            c.event::<TokenTransferFilter>();
        let topic = event.filter.topics[0].clone().unwrap();
        let topic = if let Topic::Value(v) = topic { v } else { None };
        let topic = topic.unwrap();
        info!("construct_topic {}", FixedH256(topic));
        topic
    };

    let logs = async {
        let start = 594933_u64;
        let filter = Filter::new()
            .address(c.address())
            .from_block(start)
            .to_block(start + 2);
        let logs = w3.get_logs(&filter).await?;
        for log in logs {
            // let d_log = Erc20TokenEvents::decode_log(&RawLog::from(log))?;
            // info!("{:?}", d_log);
        }
        Ok::<(), anyhow::Error>(())
    };

    async fn process_log(topic: H256, log: Log) -> anyhow::Result<()> {
        let this_topic = log.topics[0];
        info!("this_topic {:?}", this_topic);
        if topic == this_topic {
            let a = parse_log::<TokenTransferFilter>(log)?;
            info!("parse_log::<TokenTransferFilter> {:?}", a.message);
        }
        Ok(())
    }

    let tx_receipt = w3.get_transaction_receipt(transaction_hash).await?.unwrap();
    for log in tx_receipt.logs.iter() {
        let d_log = Erc20TokenEvents::decode_log(&RawLog::from(log.to_owned()))?;
        info!("Erc20TokenEvents::decode_log {:?}", d_log);
    }

    let topic = construct_topic.await;
    for log in tx_receipt.logs.into_iter() {
        let _ = process_log(topic, log).await;
    }
    info!("******");

    let _ = logs.await;
    info!("******");

    let _ = txpool.await;
    info!("******");

    let _ = decode_data.await;
    info!("******");

    let _ = read_function.await;
    info!("******");

    let _ = all_events_of_contract(&c).await;
    Ok(())
}

fn rust_file_generation() -> Result<(), Box<dyn std::error::Error>> {
    let out_file = std::env::temp_dir().join("erc20.rs");
    debug!("{:#?}", out_file.as_os_str());
    if out_file.exists() {
        std::fs::remove_file(&out_file)?;
    }
    Abigen::new("Erc20Token", ABI_PATH)?
        .generate()?
        .write_to_file(out_file)?;
    Ok(())
}

fn construct_abi() -> Result<(), Box<dyn std::error::Error>> {
    let abi_file = File::open(ABI_PATH)?;
    let reader = BufReader::new(abi_file);
    let abi = Abi::load(reader)?;
    let all_events = &abi.events;
    info!("{:?}", all_events);
    Ok(())
}

async fn all_events_of_contract(c: &Erc20Token<Provider<Http>>) {
    for x in &mut c.abi().events() {
        info!("{:?} {:?}", x.name, x.signature());
    }
}

pub trait AnyExt {
    fn type_name(&self) -> &'static str;
}

impl<T> AnyExt for T {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}
