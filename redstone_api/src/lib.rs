//! Everything related to [`Redstone`]'s api
//!
//! Will provides functions and tools to interact with Redstone's APIs
//! [`Redstone`]: https://redstone.finance/

use log::{debug, trace};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Will perform a call to a specified URL and returns a vector of [ResponseApi]
/// ToDo Make it configurable (query filter and al')
pub async fn call(url: String, assets: Vec<String>) -> Vec<ResponseApi> {
    let req_client = Client::new();
    let response = req_client.get(url).send().await.unwrap();

    let price_response: Vec<ResponseApi> = response.json().await.unwrap();

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
    async fn it_should_get_a_result() {
        let result = call("https://api.redstone.finance/prices?symbol=AVAX&provider=redstone-avalanche-prod-1&limit=1".parse().unwrap(), Vec::new()).await;
        assert_ne!(result.len(), 0)
    }
}
