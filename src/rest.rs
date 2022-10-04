use core::fmt;
use std::error::Error;
use std::fmt::{Debug, Display};
use ethers::types::Address;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

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
    pub provider_public_key: Option<String>
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RedstoneSingleTokenResponse{
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
    pub provider_public_key: Option<String>
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Source{
    pub binance: Option<f64>,
    pub binanceusdm: Option<f64>,
}
