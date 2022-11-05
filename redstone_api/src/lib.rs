//! Everything related to [`Redstone`]'s api
//!
//! Will provides functions and tools to interact with Redstone's APIs
//! [`Redstone`]: https://redstone.finance/
//!
extern crate strfmt;

use strfmt::strfmt;
use std::collections::HashMap;
use log::{debug, error, trace};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Will perform a call to a specified URL and returns a vector of [ResponseApi]
/// ToDo Make it more configurable (query filter and al')
pub async fn get_price(url: String, asset: Option<String>, provider: String) -> Vec<RedstonePriceApiResponse> {
    let mut vars = HashMap::new();
    vars.insert("provider".to_string(), provider);
    let fmt = url;
    let mut multi_asset = true;
    let is_some = asset.is_some();
    let asset = asset.unwrap_or("".to_string());
    if is_some {
        multi_asset = asset.contains(",");

        if multi_asset {
            vars.insert("symbol".to_string(), "symbols".to_string());
        } else {
            vars.insert("symbol".to_string(), "symbol".to_string());
        }
        vars.insert("assets".to_string(), asset);
    } else {
        vars.insert("symbol".to_string(), "".to_string());
        vars.insert("assets".to_string(), "".to_string());
    }

    let formatted_call = strfmt(&fmt, &vars).unwrap();
    let req_client = Client::new();
    let response = req_client.get(formatted_call).send().await.unwrap();
    let mut price_response = Vec::new();
    if multi_asset {
        let map_price_response: HashMap<String, RedstonePriceApiResponse> = response.json().await.unwrap();
        price_response = Vec::from_iter(map_price_response.values().cloned());
    } else {
        let vec_price_response: Vec<RedstonePriceApiResponse> = response.json().await.unwrap();
        price_response = vec_price_response;
    }

    // eprintln!("Raw response from Redstone Api : {:?}", price_response);
    // error!("Raw response from Redstone Api : {:?}", price_response);

    price_response
}

/// Will perform a call to a specified URL and returns a vector of [ResponseApi]
/// ToDo Make it more configurable (query filter and al')
pub async fn get_package(url: String, provider: String) -> RedstonePackageApiResponse {
    let mut vars = HashMap::new();
    vars.insert("provider".to_string(), provider);
    let fmt = url;

    let formatted_call = strfmt(&fmt, &vars).unwrap();
    let req_client = Client::new();
    let response = req_client.get(formatted_call).send().await.unwrap();
    let mut price_response = response.json().await.unwrap();

    // eprintln!("Raw response from Redstone Api : {:?}", price_response);
    // error!("Raw response from Redstone Api : {:?}", price_response);

    price_response
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RedstonePackageApiResponse {
    pub provider: Option<String>,
    pub timestamp: Option<u64>,
    #[serde(rename(deserialize = "liteSignature"))]
    pub lite_signature: Option<String>,
    pub prices: Vec<Price>,
}

/// Structure representing a response from Redstone's price API
/// ToDo Check if all fields are correctly represented
/// [`ResponseApi`]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RedstonePriceApiResponse {
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Price {
    pub _id: Option<String>,
    pub symbol: Option<String>,
    pub value: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_should_get_a_price_result_for_one_asset() {
        let result = get_price("https://api.redstone.finance/prices?{symbol}={assets}&provider={provider}&limit=1".parse().unwrap(), Some("AVAX".to_string()), "redstone-avalanche-prod-1".to_string()).await;
        assert_eq!(result.len(), 1)
    }

    #[tokio::test]
    async fn it_should_get_a_price_result_for_two_assets() {
        let result = get_price("https://api.redstone.finance/prices?{symbol}={assets}&provider={provider}&limit=1".parse().unwrap(), Some("AVAX,ETH".to_string()), "redstone-avalanche-prod-1".to_string()).await;
        println!("{:?}", result);
        assert_eq!(result.len(), 2)
    }

    #[tokio::test]
    async fn it_should_get_a_price_result_for_all_assets() {
        let result = get_price("https://api.redstone.finance/prices?{symbol}={assets}&provider={provider}&limit=1".parse().unwrap(), None, "redstone-avalanche-prod-1".to_string()).await;
        println!("{:?}", result);
        assert_eq!(result.len(), 17)
    }

    #[tokio::test]
    async fn it_should_get_a_package_result_for_one_asset() {
        let result = get_package("https://api.redstone.finance/packages/latest?provider={provider}".parse().unwrap(), "redstone-avalanche-prod-1".to_string()).await;
        println!("{:?}", result);
    }
}
