use log::{debug, trace};
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;

pub async fn call(url: String, assets: Vec<String>) ->  Vec<ResponseApi>{
    let req_client = Client::new();
    let response = req_client.get(url)
        .send().await.unwrap();

    let price_response: Vec<ResponseApi> = response.json().await.unwrap();

    price_response
}


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
    async fn it_works() {
        let result = call("https://api.redstone.finance/prices?symbol=AVAX&provider=redstone-avalanche-prod-1&limit=1".parse().unwrap(), Vec::new()).await;
    }
}
