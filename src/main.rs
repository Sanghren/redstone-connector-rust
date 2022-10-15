mod log;
mod rest;

use std::error::Error;
use std::path::Path;
use std::str::FromStr;
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
use ethers::abi::AbiEncode;
use hex::ToHex;
abigen!(ExampleContractAvalancheProd, "./abi/example_contract_avalanche_prod.abi");
use hex_literal::hex;
use reqwest::header::{CACHE_CONTROL, CONTENT_TYPE, PRAGMA, USER_AGENT};
use crate::rest::ResponseApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger("redstone_connector_rust")?;

    let anvil_instance = Anvil::at("/Users/sanghren/.foundry/bin/anvil").fork("http://10.8.0.1:9650/ext/bc/C/rpc").spawn();
    let ws = Ws::connect(anvil_instance.ws_endpoint()).await?;
    let provider = Provider::new(ws);

    let source = Path::new(&env!("CARGO_MANIFEST_DIR")).join("contracts/example-avalanche-prod-flattened.sol");
    debug!("PATH {:?}", source.as_path());
    let compiled = Solc::default().compile_source(source).expect("Could not compile contracts");
    debug!("COMPILKED {:?}", compiled);
    debug!("A");
    let (abi, bytecode, _runtime_bytecode) =
        compiled.find("ExampleContractAvalancheProd").expect("could not find contract").into_parts_or_default();
    debug!("B");
    debug!("ABI {:?}", abi);
    debug!("BYTECODE {:?}", bytecode);
    debug!("RUNTIME BYTECODE {:?}", _runtime_bytecode);
    // let compiled = Solc::default().compile_source("../contracts/example-avalanche-prod-flattened.sol").unwrap();
    // debug!("{:?}", compiled);
    let contract = compiled
        .get("/Users/sanghren/Documents/PerSpace/Codespace/redstone-connector-rust/contracts/example-avalanche-prod-flattened.sol", "ExampleContractAvalancheProd")
        .expect("could not find contract");

    // 2. instantiate our wallet
    let wallet: LocalWallet = anvil_instance.keys()[0].clone().into();
    debug!("C");
    debug!("Wallet address {:?}", wallet);
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

    let req_client = reqwest::Client::new();
    let response = req_client.get("https://api.redstone.finance/prices?symbol=AVAX&provider=redstone-avalanche-prod-1&limit=1")
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 12_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36")
        .header(CONTENT_TYPE, "application/json")
        .header(CACHE_CONTROL, "no-store")
        .header(PRAGMA, "no-cache")
        .send().await?;

    let price_response: Vec<ResponseApi> = response.json().await?;
    trace!("Raw JSON response for ResponseApi<TeamResult>: {:?}", price_response);

    // Prepare the SerializedData
    let mut serialized_data = SerializedPriceData {
        symbols: vec![],
        values: vec![],
        timestamp: 0,
        lite_sig: String::new(),
    };

    serialized_data.timestamp = price_response.get(0).unwrap().timestamp.unwrap();
    serialized_data.symbols.push(price_response.get(0).unwrap().symbol.clone().unwrap());
    let vv = (price_response.get(0).unwrap().value.unwrap() * 1000000.) as u64;
    // let vv = 1603300000;
    serialized_data.values.push(vv);
    serialized_data.lite_sig = price_response.get(0).unwrap().lite_evm_signature.clone().unwrap();

    let data_to_append = get_lite_data_bytes_string(serialized_data);

    // ToDo Check this selector , should be dynamic of course but here want to test the setprice one
    let mut data = String::from("da93d0d1");
    data += &*data_to_append;

    println!("After appending vanilla and generated data -- {:?}", data);
    // println!("After appending vanilla and generated dataWW -- {:?}", hex::decode(data).unwrap());

    let res = instance.get_last_price().call().await?;
    debug!("GET LAST PRICE BEOFRE SETTING IT {:?}", res);

    // Now we should be good to send the tx.
    let tx = TransactionRequest::new()
        .to(contract.address())
        .data(Bytes::from(hex!("da93d0d14156415800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f8bd6c0000000000000000000000000000000000000000000000000000000006345510a0156009cdfb27d3270b0cb427398233f3a9621b55517c3cc0f71ab16b9e6e09fcc7f9ad594cb21f5a235c73e6e5ce8d13cbd8b957153d65187ffe919414d0486751b")))
        // .data(Bytes::from(hex!("da93d0d141564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063b2a06c000000000000000000000000000000000000000000000000000000006343ed3b01b5cd0cabe77e0690352a1049316d942813ae50f59d3e55f892773151494a8a4957fba2e5b28eb904567c20ec530f9dd6532bae9a66250e5417c9b74fbe2046091c")))
        .chain_id(43114);

    debug!("Attempt tx {:?}", tx);
    let receipt =
        client.clone().send_transaction(tx, None).await.unwrap().await.unwrap().unwrap();
    debug!("ATCHOUM - {:?}", receipt);

    let res = instance.get_last_price().call().await?;
    debug!("GET LAST PRICE AFTER SETTING IT {:?}", res);

    Ok(())
}

pub fn compile_contract(name: &str, filename: &str) -> (Abi, Bytes) {
    let path = format!("./tests/solidity-contracts/{}", filename);
    let compiled = Solc::default().compile_source(&path).unwrap();
    let contract = compiled.get(&path, name).expect("could not find contract");
    let (abi, bin, _) = contract.into_parts_or_default();
    (abi, bin)
}
// 4156415800000000000000000000000000000000000000000000000000000000

pub fn get_lite_data_bytes_string(price_data: SerializedPriceData) -> String {
    let mut data = String::new();

    for (index, symbol) in price_data.symbols.into_iter().enumerate() {
        let symbol = symbol;
        let value = price_data.values.get(index).unwrap();
        // let value = 1603000000;
        let b32 = ethers::utils::format_bytes32_string(&*symbol).unwrap();
        let b32_hex = b32.encode_hex();
        let b32_hex_stripped = b32_hex.strip_prefix("0x").unwrap();
        data += b32_hex_stripped;
        data += value.encode_hex().strip_prefix("0x").unwrap();


        let timestamp = price_data.timestamp / 1000;
        // let timestamp = 1665487114642_u64 / 1000;
        let timestamp_hex = timestamp.encode_hex();
        let timestamp_hex_stripped = timestamp_hex.strip_prefix("0x").unwrap();

        data += timestamp_hex_stripped;

        let len_hex = format!("{:#04x}", price_data.values.len());
        let len_hex = len_hex.strip_prefix("0x").unwrap();

        data += len_hex;

        let lite_sig = price_data.lite_sig.clone();
        let lite_sig = lite_sig.strip_prefix("0x").unwrap();

        data += lite_sig;

        println!("OYYYYYYH {:02X?}", ethers::utils::format_bytes32_string(&*symbol).unwrap().encode_hex().strip_prefix("0x"));
        println!("OYYYYYYH - 2 {:?}", value.encode_hex().strip_prefix("0x"));
        println!("OYYYYYYH - 2 {:?}", data);
    }

    data
}


// ToDo Soooo we will append the data we generated (see below) to the "vanilla" tx.data
//         4156415800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f8bd6c0000000000000000000000000000000000000000000000000000000006345510a0182d530263f8c2c6f8280187f98b74f3788f8dcc816972558cee07a3cad4926fb69da82d4c97092347ac1a4df6481b953c7b97f974fc79a38ad98b7742f3fddd71c
// da93d0d141564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063b2a06c000000000000000000000000000000000000000000000000000000006343ed3b01b5cd0cabe77e0690352a1049316d942813ae50f59d3e55f892773151494a8a4957fba2e5b28eb904567c20ec530f9dd6532bae9a66250e5417c9b74fbe2046091c
// da93d0d14156415800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f8bd6c0000000000000000000000000000000000000000000000000000000006345510a018b6c20ee5dbe4c970cee4155a522511edc684e43b4ac835d48b883530a33f3bc2a6a1f95e4e24252ce36a2ba8dcba1e1db68d3c23ccb0cd7025e2da5be6129a31b
//         4156415800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f8bd6c0000000000000000000000000000000000000000000000000000000006345510b01dc6f1c3318f59302089722e3f98d17b8ca43f4e56b345066dcf2915f0c2b6a8553dc8d86b6be7c1d378a4054a093d5b83fca13863c81987c83137b2448f203b11c
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


pub struct SerializedPriceData {
    symbols: Vec<String>,
    values: Vec<u64>,
    timestamp: u64,
    lite_sig: String,
}

// export interface SerializedPriceData {
// symbols: string[];
// values: any[];
// timestamp: number;
// }