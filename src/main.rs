#![allow(unused_variables, dead_code)]

use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;

use chrono::Local;
use env_logger::fmt::style::Color;
use ethers::abi::{Abi, AbiDecode, Detokenize, InvalidOutputType, RawLog, Token};
use ethers::prelude::rand::SeedableRng;
use ethers::prelude::*;
use ethers::utils::{hex, parse_checksummed, to_checksum};
use log::{info, warn, Level, LevelFilter};
use std::sync::OnceLock;
use crate::erc20::*;
use crate::setting::Setting;

mod erc20;
mod setting;

static CHAIN_ID: OnceLock<U256> = OnceLock::new();
static SETTING: OnceLock<Setting> = OnceLock::new();

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
        if let [Token::Address(to), Token::Uint(value), Token::String(data)] = &tokens[..] {
            let c = serde_json::from_str(data).map_err(|_| e.clone())?;
            Ok(Self {
                to: *to,
                value: *value,
                data: c,
            })
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
    let setting=SETTING.get_or_init(Setting::init);
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .format(|buf, record| {
            let mut level_style = buf.default_level_style(record.level());
            let reset = level_style.render_reset();
            if record.level() == Level::Warn {
                level_style = level_style.fg_color(Some(Color::Ansi256(206_u8.into())));
            }
            let level_style = level_style.render();
            writeln!(
                buf,
                "{level_style}[{} | line:{:<4}|{}]: {}{reset}",
                Local::now().format("%H:%M:%S"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .init();

    let w3: Arc<Provider<Http>> = Arc::new(Provider::<Http>::try_from(&setting.rpc)?);
    let c: Erc20Token<Provider<Http>> =
        Erc20Token::new(setting.token.parse::<Address>()?, w3.clone());
    let transaction_hash: TxHash =
        "0x079b409e03acb6b2f9985032de5325aaef242336c3eef2f007c32102a23fb66c".parse()?;

    warn!("{}", format!("{:?}", transaction_hash));
    warn!("{}", FixedH256(transaction_hash));

    // 设置全局变量
    let chain_id = w3.get_chainid().await?;
    CHAIN_ID
        .set(chain_id)
        .unwrap_or_else(|_| panic!("can't set chain_id"));

    let mut rng = rand::rngs::StdRng::from_entropy();
    let new_wallet = LocalWallet::new(&mut rng);
    let _ =
        LocalWallet::from_str("c10fde8c099dd7e3b44a0154bf083180572068f1c2572bbd3985558fe1ff821b")?;
    let address = to_checksum(&new_wallet.address(), None);
    let key = hex::encode(new_wallet.signer().to_bytes());
    info!("address={}, key={}", address, key);

    info!("{:*^80}", "get chains's txpool");
    let txpool = async {
        let txpool = w3.txpool_content().await?;
        for (addr, txs) in txpool.pending {
            info!("txpool {}", format!("{:?}", addr));
            for (nonce, tx) in txs {}
        }
        Ok::<(), anyhow::Error>(())
    };
    let _ = txpool.await;

    info!("{:*^80}", "decode_input");
    let decode_data = async {
        let tx = w3.get_transaction(transaction_hash).await?.unwrap();
        let decode_input2 = Erc20TokenCalls::decode(&tx.input)?;
        if let Erc20TokenCalls::TokenTransfer(v) = decode_input2 {
            info!("decode_data2 {:?}", v)
        };
        let decode_input3 = c.decode_input::<TokenTransferCall, Bytes>(tx.input.clone())?;
        info!("decode_data3 {:?}", decode_input3);
        let decode_input4 = c.decode_input::<Input, Bytes>(tx.input)?;
        info!("decode_data4 {:?}", decode_input4);
        Ok::<(), anyhow::Error>(())
    };
    let _ = decode_data.await;

    info!("{:*^80}", "call function");
    let call_function = async {
        let aa = c
            .balance_of(parse_checksummed(
                "0x44ea38b427fce87147dc034caae56f4a46cdfe98",
                None,
            )?)
            .call()
            .await?;
        info!("read_function {:?}", aa);
        Ok::<(), anyhow::Error>(())
    };
    let _ = call_function.await;

    let event_topic2 = async {
        let event = c.event::<TokenTransferFilter>();
        let topic = event.filter.topics[0].clone().unwrap();
        if let Topic::Value(v) = topic {
            v
        } else {
            None
        }
    };

    let event_topic = TokenTransferFilter::signature();
    let event_name = TokenTransferFilter::name();
    let event_instance = c.token_transfer_filter();
    let event_instance2 = c.event::<TokenTransferFilter>();
    let event_instance3 = Contract::event_of_type::<TokenTransferFilter>(w3.clone());
    let topic2 = event_topic2.await.unwrap();
    assert_eq!(topic2, event_topic);

    info!("{:*^80}", "query_all_events");
    async fn query_events(contract: &Erc20Token<Provider<Http>>) -> Result<(), anyhow::Error> {
        let start = 594933_u64;
        let events = contract.events().from_block(start).to_block(start + 2);
        let logs = events.query_with_meta().await?;
        for (decoded_log, meta) in logs {
            if let Erc20TokenEvents::TokenTransferFilter(f) = decoded_log {
                info!("{f:?}");
            }
        }
        Ok(())
    }
    let _ = query_events(&c).await;

    async fn query_specific_events(
        contract: &Erc20Token<Provider<Http>>,
    ) -> Result<(), anyhow::Error> {
        let start = 594933_u64;
        let events = contract
            .event::<TokenTransferFilter>()
            .from_block(start)
            .to_block(start + 2);
        let logs: Vec<(TokenTransferFilter, LogMeta)> = events.query_with_meta().await?;
        for (decoded_log, meta) in logs {
            info!("{decoded_log:?}");
        }
        Ok(())
    }
    let _ = query_specific_events(&c).await;

    let query_events_from_rpc_api = async {
        let start = 594933_u64;
        let filter = Filter::new()
            .address(c.address())
            .from_block(start)
            .to_block(start + 2);
        let logs = w3.get_logs(&filter).await?;
        for log in logs {
            let decoded_log = Erc20TokenEvents::decode_log(&RawLog::from(log))?;
            if let Erc20TokenEvents::TokenTransferFilter(f) = decoded_log {
                info!("{f:?}");
            }
        }
        Ok::<(), anyhow::Error>(())
    };

    info!(
        "{:*^80}",
        "process receipt logs with Erc20TokenEvents::decode_log"
    );
    let tx_receipt = w3.get_transaction_receipt(transaction_hash).await?.unwrap();
    for log in tx_receipt.logs.iter() {
        let decoded_log = Erc20TokenEvents::decode_log(&RawLog::from(log.to_owned()))?;
        if let Erc20TokenEvents::TokenTransferFilter(f) = decoded_log {
            info!("{f:?}");
        }
    }

    info!(
        "{:*^80}",
        "process receipt logs with parse_log::<TokenTransferFilter>"
    );
    for log in tx_receipt.logs.into_iter() {
        if event_topic == log.topics[0] {
            // let decoded_log =
            //     <TokenTransferFilter as EthLogDecode>::decode_log(&RawLog::from(log.to_owned()))?;
            let decoded_log: TokenTransferFilter = parse_log::<TokenTransferFilter>(log)?;
            info!("parse_log::<TokenTransferFilter> {:?}", decoded_log)
        }
    }

    let _ = all_events_of_contract(&c).await;
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
