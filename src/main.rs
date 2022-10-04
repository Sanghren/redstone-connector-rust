mod log;
mod rest;

use std::error::Error;
use std::path::Path;
use ::log::{debug, error, trace};
use ethers::prelude::*;
use ethers::utils::Anvil;
use ethers_providers::{Provider, Ws};
use crate::log::setup_logger;
use ethers_contract::{abigen, ContractFactory, EthAbiType};
use crate::abi::Abi;
use ethers_solc::Solc;
use tokio::time::{sleep, Duration};
use std::sync::Arc;
abigen!(ExampleContractAvalancheProd, "./abi/example_contract_avalanche_prod.abi");
use hex_literal::hex;
use reqwest::header::{CACHE_CONTROL, CONTENT_TYPE, PRAGMA, USER_AGENT};
use crate::rest::ResponseApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger("redstone_connector_rust")?;
    let anvil_instance = Anvil::at("/home/tbrunain/.foundry/bin/anvil").fork("http://10.8.0.1:9650/ext/bc/C/rpc").spawn();
    let ws = Ws::connect(anvil_instance.ws_endpoint()).await?;
    let provider = Provider::new(ws);
    let source = Path::new(&env!("CARGO_MANIFEST_DIR")).join("contracts/example-avalanche-prod-flattened.sol");
    debug!("PATH {:?}", source.as_path());
    let compiled = Solc::default().compile_source(source).expect("Could not compile contracts");
    // debug!("COMPILKED {:?}", compiled);
    debug!("A");
    let (abi, bytecode, _runtime_bytecode) =
        compiled.find("ExampleContractAvalancheProd").expect("could not find contract").into_parts_or_default();
    debug!("B");
    // let compiled = Solc::default().compile_source("../contracts/example-avalanche-prod-flattened.sol").unwrap();
    // let contract = compiled
    //     .get("../contracts/example-avalanche-prod-flattened.sol", "ExampleContractAvalancheProd")
    //     .expect("could not find contract");
    //
    // let (abi, bytecode, _runtime_bytecode) =
    //     compiled.find("AvalancheProd").expect("could not find contract").into_parts_or_default();
    //

    // 2. instantiate our wallet
    let wallet: LocalWallet = anvil_instance.keys()[0].clone().into();
    debug!("C");
    // 3. connect to the network
    let provider =
        Provider::<Http>::try_from(anvil_instance.endpoint())?.interval(Duration::from_millis(10u64));
    debug!("D");
    // 4. instantiate the client with the wallet
    let client = SignerMiddleware::new(provider, wallet.with_chain_id(43114 as u64));
    let client = Arc::new(client);
    debug!("E");
    // 5. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(abi, bytecode, client.clone());
    debug!("F");
    // 6. deploy it with the constructor arguments
    let contract = factory.deploy(())?.send().await?;
    debug!("Address {:?}", contract.address());
    debug!("Method {:?}", contract);
    let instance = ExampleContractAvalancheProd::new(contract.address(), client.clone());

    // Data here is crafted from redstone connector . I just copy pasted the data generate by the ts lib.
    // It is timestamped
    // let tx = TransactionRequest::new()
    //     .to(contract.address())
    //     //ToDo How to craft this data payload
    //     .data(Bytes::from(hex!("da93d0d1415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000635a5dee00000000000000000000000000000000000000000000000000000000633aa2ae018c5178cbda618691087fd972b2990c24c0c21340869f5b703122287de0ad2d6517e4570385332705ee8a81debe377c97a6490e35fa1608bd24af25bf8e2d09281c")))
    //                                                                                                                                                                                                                                       // 140c2edbb7f39397c5a9d6a25cb488e6db65f5befdfe92467997990ba5d249e9452f3e40eac910cc6e95039f3eb7f6c290e20c111fbf9e7f55a30bdc7732cf2e1b
    //     .chain_id(43114);
    // let receipt =
    //     client.clone().send_transaction(tx, None).await.unwrap().await.unwrap().unwrap();
    // debug!("ATCHOUM - {:?}", receipt);
    //
    // let res = instance.get_last_price().call().await?;
    // debug!("res {:?}", res);
    // trace!("Logger initialized");

    let client = reqwest::Client::new();
    let response = client.get("https://api.redstone.finance/prices/?symbol=AVAX&provider=redstone&limit=1")
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 12_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36")
        .header(CONTENT_TYPE, "application/json")
        .header(CACHE_CONTROL, "no-store")
        .header(PRAGMA, "no-cache")
        .send().await?;

    let price_response: Vec<ResponseApi> = response.json().await?;
    trace!("Raw JSON response for ResponseApi<TeamResult>: {:?}", price_response);

    Ok(())
}

pub fn compile_contract(name: &str, filename: &str) -> (Abi, Bytes) {
    let path = format!("./tests/solidity-contracts/{}", filename);
    let compiled = Solc::default().compile_source(&path).unwrap();
    let contract = compiled.get(&path, name).expect("could not find contract");
    let (abi, bin, _) = contract.into_parts_or_default();
    (abi, bin)
}

pub fn get_lite_data_bytes_string(price_data: SerializedPriceData) -> String {
    let mut data = String::new();

    for (index, symbol) in price_data.symbols.into_iter().enumerate() {
        let symbol = symbol;
        let value = price_data.values.get(index).unwrap();
        let mut b32 = format!("{:?}", ethers::utils::format_bytes32_string(&*symbol).unwrap());
        data += &*b32
    }

    data
}



// getLiteDataBytesString(priceData: SerializedPriceData): string {
// // Calculating lite price data bytes array
// let data = "";
// for (let i = 0; i < priceData.symbols.length; i++) {
// const symbol = priceData.symbols[i];
// const value = priceData.values[i];
// data += symbol.substr(2) + value.toString(16).padStart(64, "0");
// }
// data += Math.ceil(priceData.timestamp / 1000)
// .toString(16)
// .padStart(64, "0");
//
// return data;
// }


struct SerializedPriceData {
    symbols: Vec<String>,
    values: Vec<u64>,
    timestamp: u64
}

// export interface SerializedPriceData {
// symbols: string[];
// values: any[];
// timestamp: number;
// }