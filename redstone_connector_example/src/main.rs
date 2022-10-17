mod log;
mod rest;
use dotenv::dotenv;
#[macro_use]
extern crate dotenv_codegen;
use crate::{abi::Abi, log::setup_logger};
use ::log::{debug, error, trace};
use ethers::{abi::AbiEncode, prelude::*, utils::Anvil};
use ethers_contract::{abigen, ContractFactory, EthAbiType};
use ethers_providers::{Provider, Ws};
use ethers_solc::Solc;
use hex::ToHex;
use std::{error::Error, path::Path, str::FromStr, sync::Arc};
use tokio::time::{sleep, Duration};
abigen!(ExampleContractAvalancheProd, "./abi/example_contract_avalanche_prod.abi");
use crate::rest::ResponseApi;
use hex_literal::hex;
use redstone_connector_rust::add_redstone_data;
use reqwest::header::{CACHE_CONTROL, CONTENT_TYPE, PRAGMA, USER_AGENT};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger("redstone_connector_example")?;

    let anvil_instance = Anvil::at(dotenv!("ANVIL_PATH")).fork(dotenv!("RPC_ENDPOINT")).spawn();
    let ws = Ws::connect(anvil_instance.ws_endpoint()).await?;
    let provider = Provider::new(ws);

    let source = Path::new(&env!("CARGO_MANIFEST_DIR"))
        .join("contracts/example-avalanche-prod-flattened.sol");
    debug!("PATH {:?}", source.as_path());
    let compiled = Solc::default().compile_source(source).expect("Could not compile contracts");
    debug!("COMPILKED {:?}", compiled);
    debug!("A");
    let (abi, bytecode, _runtime_bytecode) = compiled
        .find("ExampleContractAvalancheProd")
        .expect("could not find contract")
        .into_parts_or_default();
    debug!("B");
    // let compiled =
    let contract = compiled
        .get(
            Path::new(&env!("CARGO_MANIFEST_DIR"))
                .join("contracts/example-avalanche-prod-flattened.sol")
                .as_path()
                .to_str()
                .unwrap(),
            "ExampleContractAvalancheProd",
        )
        .expect("could not find contract");

    // 2. instantiate our wallet
    let wallet: LocalWallet = anvil_instance.keys()[0].clone().into();
    debug!("C");
    debug!("Wallet address {:?}", wallet);
    // 3. connect to the network
    let provider = Provider::<Http>::try_from(anvil_instance.endpoint())?
        .interval(Duration::from_millis(10u64));
    debug!("D");
    // 4. instantiate the client with the wallet
    let client = SignerMiddleware::new(
        provider,
        wallet.with_chain_id(u64::from_str(dotenv!("CHAIN_ID")).unwrap()),
    );
    let client = Arc::new(client);
    debug!("E");
    // 5. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(abi, bytecode, client.clone());
    debug!("F");
    // 6. deploy it with the constructor arguments
    let contract = factory.deploy(())?.send().await?;
    debug!("Address {:?}", contract.address());
    // debug!("Method {:?}", contract);

    let instance = ExampleContractAvalancheProd::new(contract.address(), client.clone());

    // Data here is crafted from redstone connector . I just copy pasted the data generate by the ts
    // lib. It is timestamped
    let mut data = String::from("da93d0d1");
    data = add_redstone_data(data, ["AVAX".to_string()].to_vec()).await;

    println!("After appending vanilla and generated data -- {:?}", data);

    let res = instance.get_last_price().call().await?;
    debug!("GET LAST PRICE BEOFRE SETTING IT {:?}", res);

    // Now we should be good to send the tx.
    let tx = TransactionRequest::new()
        .to(contract.address())
        .data(Bytes::from_str(data.as_str()).unwrap())
        .chain_id(43114);

    debug!("Attempt tx {:?}", tx);
    let receipt = client.clone().send_transaction(tx, None).await.unwrap().await.unwrap().unwrap();
    debug!("ATCHOUM - {:?}", receipt);

    let res = instance.get_last_price().call().await?;
    debug!("GET LAST PRICE AFTER SETTING IT {:?}", res);

    Ok(())
}
