mod log;

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
// abigen!(ExampleContractAvalancheProd, "./abi/example-avalanche-prod-flattened.sol.abi");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger("redstone_connector_rust")?;
    let anvil_instance = Anvil::at("/home/xxx/.foundry/bin/anvil").fork("http://10.8.0.1:9650/ext/bc/C/rpc").spawn();
    let ws = Ws::connect(anvil_instance.ws_endpoint()).await?;
    let provider = Provider::new(ws);
    let source = Path::new(&env!("CARGO_MANIFEST_DIR")).join("contracts/example-avalanche-prod-flattened.sol");
    debug!("PATH {:?}", source.as_path());
    let compiled = Solc::default().compile_source(source).expect("Could not compile contracts");
    debug!("COMPILKED {:?}", compiled);
    let (abi, bytecode, _runtime_bytecode) =
        compiled.find("ExampleContractAvalancheProd").expect("could not find contract").into_parts_or_default();
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

    // 3. connect to the network
    let provider =
        Provider::<Http>::try_from(anvil_instance.endpoint())?.interval(Duration::from_millis(10u64));

    // 4. instantiate the client with the wallet
    let client = SignerMiddleware::new(provider, wallet.with_chain_id(43114 as u64));
    let client = Arc::new(client);

    // 5. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(abi, bytecode, client.clone());

    // 6. deploy it with the constructor arguments
    let contract = factory.deploy("initial value".to_string())?.send().await?;

    trace!("Logger initialized");
    Ok(())
}

pub fn compile_contract(name: &str, filename: &str) -> (Abi, Bytes) {
    let path = format!("./tests/solidity-contracts/{}", filename);
    let compiled = Solc::default().compile_source(&path).unwrap();
    let contract = compiled.get(&path, name).expect("could not find contract");
    let (abi, bin, _) = contract.into_parts_or_default();
    (abi, bin)
}