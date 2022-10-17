//! Everything related to [`Redstone`]'s api
//!
//! Will provides functions and tools to interact with Redstone's APIs
//! [`Redstone`]: https://redstone.finance/
//!
extern crate strfmt;
use strfmt::strfmt;
use std::collections::HashMap;
use log::{debug, trace};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Will perform a call to a specified URL and returns a vector of [ResponseApi]
/// ToDo Make it more configurable (query filter and al')
pub async fn get_price(url: String, asset: String) -> Vec<ResponseApi> {
    let mut vars = HashMap::new();
    let multi_asset = asset.contains(",");
    let fmt = url;
    if multi_asset {
        vars.insert("symbol".to_string(), "symbols");
    } else {
        vars.insert("symbol".to_string(), "symbol");
    }
    vars.insert("assets".to_string(), asset.as_str());

    let formatted_call = strfmt(&fmt, &vars).unwrap();
    let req_client = Client::new();
    let response = req_client.get(formatted_call).send().await.unwrap();
    let mut price_response = Vec::new();
    if multi_asset {
        let map_price_response: HashMap<String,ResponseApi> = response.json().await.unwrap();
        price_response = Vec::from_iter(map_price_response.values().cloned());
    } else {
        let vec_price_response: Vec<ResponseApi> = response.json().await.unwrap();
        price_response = vec_price_response;
    }

    price_response
}

/// Structure representing a response from Redstone's price API
/// ToDo Check if all fields are correctly represented
/// [`ResponseApi`]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResponseApi {
    pub id: Option<String>,
    pub symbol: Option<String>,
    pub provider: Option<String>,
    pub value: Option<f64>,
    #[serde(rename(deserialize = "liteEvmSignature"))]
    pub lite_evm_signature: Option<String>,
    #[serde(rename(deserialize = "permawebTx"))]
    pub permaweb_tx: Option<String>,
    pub version: Option<String>,
    pub source: Option<Source>,
    pub timestamp: Option<u64>,
    pub minutes: Option<u64>,
    #[serde(rename(deserialize = "providerPublicKey"))]
    pub provider_public_key: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RedstoneSingleTokenResponse {
    pub id: Option<String>,
    pub symbol: Option<String>,
    pub provider: Option<String>,
    pub value: Option<f64>,
    #[serde(rename(deserialize = "liteEvmSignature"))]
    pub lite_evm_signature: Option<String>,
    #[serde(rename(deserialize = "permawebTx"))]
    pub permaweb_tx: Option<String>,
    pub version: Option<String>,
    pub source: Option<String>,
    pub timestamp: Option<u64>,
    pub minutes: Option<u64>,
    #[serde(rename(deserialize = "providerPublicKey"))]
    pub provider_public_key: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Source {
    pub binance: Option<f64>,
    #[serde(rename(deserialize = "binanceusdm"))]
    pub binance_usdm: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_should_get_a_result_for_one_asset() {
        let result = get_price("https://api.redstone.finance/prices?{symbol}={assets}&provider=redstone-avalanche-prod-1&limit=1".parse().unwrap(), "AVAX".to_string()).await;
        assert_eq!(result.len(), 1)
    }

    #[tokio::test]
    async fn it_should_get_a_result_for_two_assets() {
        let result = get_price("https://api.redstone.finance/prices?{symbol}={assets}&provider=redstone-avalanche-prod-1&limit=1".parse().unwrap(), "AVAX,ETH".to_string()).await;
        println!("{:?}", result);
        assert_eq!(result.len(), 2)
    }
}
