//! Will provide means to use Redstone in Rust
//!
//! Will provides functions to interact with Redstone's
//! [`Redstone`]: https://redstone.finance/
use base64::prelude::*;
use rustc_hex::ToHex;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fmt::Debug;
use std::time::Duration;
use base64::{alphabet, encode, engine};
use base64::alphabet::Alphabet;
use ethers::abi::AbiEncode;
use ethers::utils::__serde_json::to_vec;
use ethers::utils::{format_bytes32_string, hex};
use log::{debug, error, info, trace};
use redstone_api::{get_package, get_price};
use data_encoding::BASE64;
use data_encoding::HEXLOWER;

/// Function that will add at the end of the data the redstone specific data that we will craft
/// It returns the data it got as input + extra, where extra is generated following redstone logic
pub async fn get_prices(data: String, vec_assets: Vec<&str>, provider: String, vec_token_order: Vec<&str>) -> String {
    let mut assets: Option<String> = Some(String::new());
    if vec_assets.is_empty() {
        assets = None;
    }
    let vec_len = vec_assets.len();
    for asset in vec_assets {
        let mut assetss = assets.unwrap();
        assetss += asset;
        if vec_len > 1 {
            assetss += ",";
        }

        assets = Some(assetss);
    }

    //ToDo Rename this
    let vec_response_api = redstone_api::get_prices("https://api.redstone.finance/prices?provider={provider}&symbols={assets}".parse().unwrap(), assets, provider).await;

    let mut serialized_data = SerializedPriceData {
        map_symbol_value: BTreeMap::new(),
        timestamp: 0,
        lite_sig: String::new(),
    };

    serialized_data.timestamp = vec_response_api.get(0).unwrap().timestamp.unwrap();
    serialized_data.lite_sig = vec_response_api.get(0).unwrap().lite_evm_signature.clone().unwrap();
    for r in vec_response_api {
        serialized_data.map_symbol_value.insert(r.symbol.unwrap(), r.value.unwrap());
        // serialized_data.symbols.push(r.symbol.unwrap());
        // serialized_data.values.push((r.value.unwrap() * 100000000.).round() as u64);
    }

    // ToDo It must work for an array with more than 1 asset
    // serialized_data.symbols.push(vec_response_api.get(0).unwrap().symbol.clone().unwrap());
    // let value = (vec_response_api.get(0).unwrap().value.unwrap() * 100000000.) as u64;
    // serialized_data.values.push(value);
    let data_to_append = get_lite_data_bytes_string(serialized_data);

    // append the result of the above line to input data
    let new_data = data + &*data_to_append;
    // return the whole things
    new_data
}

/// Function that will add at the end of the data the redstone specific data that we will craft
/// It returns the data it got as input + extra, where extra is generated following redstone logic
pub async fn get_packages(data: String, number_of_data_package: usize, order_of_assets: Vec<String>, data_feeds: Vec<String>) -> String {
    let data_feeds_ids = if data_feeds.is_empty() { ["___ALL_FEEDS___".to_string()].to_vec() } else { data_feeds };

    //ToDo Rename this
    let map_response_api = get_package("https://oracle-gateway-2.a.redstone.finance/data-packages/latest/redstone-avalanche-prod".parse().unwrap()).await;

    let mut i = 0_usize;
    let mut new_data = data;

    // order of assets ....
    while i < number_of_data_package {
        let mut serialized_data = SerializedPriceData {
            map_symbol_value: BTreeMap::new(),
            timestamp: 0,
            lite_sig: String::new(),
        };

        for asset in &order_of_assets {
            serialized_data.timestamp = map_response_api.get("___ALL_FEEDS___").unwrap().get(i).unwrap().timestampMilliseconds as u64;
            serialized_data.lite_sig = map_response_api.get("___ALL_FEEDS___").unwrap().get(i).unwrap().signature.clone();

            println!("Key {}", asset);
            for data_point in &map_response_api.get("___ALL_FEEDS___").unwrap().get(i).unwrap().dataPoints {
                if asset.eq_ignore_ascii_case(&data_point.dataFeedId) {
                    serialized_data.map_symbol_value.insert(asset.clone(), data_point.value);
                }
            }
            // for r in &map_response_api {
            //     serialized_data.map_symbol_value.insert(r.0.clone(), (r.1.get(i).unwrap().dataPoints.get(0).unwrap().value * 100000000.).round() as u64);
            //     // serialized_data.symbols.push(r.symbol.unwrap());
            //     // serialized_data.values.push((r.value.unwrap() as u128 * 100000000.).round() as u64);
            // }
            // ToDo It must work for an array with more than 1 asset
            // serialized_data.symbols.push(vec_response_api.get(0).unwrap().symbol.clone().unwrap());
            // let value = (vec_response_api.get(0).unwrap().value.unwrap() * 100000000.) as u64;
            // serialized_data.values.push(value);
        }
        let data_to_append = get_lite_data_bytes_string(serialized_data);
        new_data += &*data_to_append;
        i += 1;
    }


    // append the result of the above line to input data

    add_meta_data_bytes(&mut new_data);

    // return the whole things
    new_data
}

pub fn get_lite_data_bytes_string(price_data: SerializedPriceData) -> String {
    let mut data = String::new();
    let len_map = price_data.map_symbol_value.len();
    for (symbol, value) in price_data.map_symbol_value.iter() {
        let symbol = symbol;
        trace!("Processing information about {:?}", symbol);
        let b32 = ethers::utils::format_bytes32_string(&*symbol).unwrap();
        let b32_hex = b32.encode_hex();
        let b32_hex_stripped = b32_hex.strip_prefix("0x").unwrap();
        data += b32_hex_stripped;
        let engine = engine::GeneralPurpose::new(&alphabet::URL_SAFE,
            engine::general_purpose::NO_PAD,
        );
        let test = engine.encode(&value.to_be_bytes());
        // let test = Engine::encode(&value.to_be_bytes(), Engine::);
        println!("TEST {}", test);
        data += test.as_str();
    }
    let timestamp = price_data.timestamp as u64;
    // let tmstmp = Duration::from_secs(timestamp);
    println!("Timestamp : {:?}", timestamp);
    let timestamp_hex = format!("{:#04x}", timestamp);
    println!("Timestamp : {:?}", timestamp_hex);
    let timestamp_hex_stripped = timestamp_hex.strip_prefix("0x").unwrap();
    println!("Timestamp : {:?}", timestamp_hex_stripped);
    data += "0";
    data += timestamp_hex_stripped;

    let data_point_size_hex = format!("{:0>8x}", 32);
    println!("data_point_size_hex : {:?}", data_point_size_hex);

    data += data_point_size_hex.as_str();

    // ToDo Automatic and not hardcoded
    let data_point_number_hex = format!("{:0>6x}", 34);
    println!("data_point_number_hex : {:?}", data_point_number_hex);

    data += data_point_number_hex.as_str();

    println!("{}", price_data.lite_sig.clone());
    // Decode the Base64 string
    let lite_sig = price_data.lite_sig.clone();

    let decoded = BASE64.decode(lite_sig.as_bytes()).unwrap();

    // Encode the decoded bytes as a hexadecimal string
    let hex_string = HEXLOWER.encode(&decoded);

    println!("{}", hex_string);
    // let bytes32 = hex::encode(lite_sig.as_bytes());
    // let lite_sig = format!("{:04x}", bytes32);
    // println!("{}", bytes32);
    let lite_sig = hex_string.trim_start_matches("0x");

    data += lite_sig;


    data
}

fn add_meta_data_bytes(data: &mut String) {
    // ToDo Dynamic
    let package_number_hex = format!("{:0>4x}", 3);

    *data += package_number_hex.as_str();

    let b32 = ethers::utils::format_bytes32_string("0.0.19#redstone-avalanche-prod").unwrap();
    let b32_hex = b32.encode_hex();
    let b32_hex_stripped = b32_hex.strip_prefix("0x").unwrap();

    *data += b32_hex_stripped;

    let metadata_size_hex = format!("{:#04x}", 30);
    let metadata_size_hex = metadata_size_hex.strip_prefix("0x").unwrap();

    *data += metadata_size_hex;

    *data += "000002ed57011e0000";
}

fn string_to_bytes32(s: &str) -> String {
    let mut bytes = [0u8; 32];
    let string_bytes = s.as_bytes();

    for (i, byte) in string_bytes.iter().enumerate() {
        if i >= 32 {
            break;
        }
        bytes[i] = *byte;
    }

    match std::str::from_utf8(&bytes) {
        Ok(s) => String::from(s.trim_end_matches('\0')),
        Err(_) => String::new(),
    }
}

#[derive(Debug)]
pub struct SerializedPriceData {
    map_symbol_value: BTreeMap<String, f64>,
    timestamp: u64,
    lite_sig: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works_for_one_asset() {
        let result = get_prices("".parse().unwrap(), ["AVAX"].to_vec(), "redstone-avalanche-prod-1".to_string(), [
            "AVAX",
            "BTC",
            "ETH",
            "FRAX",
            "LINK",
            "MOO_TJ_AVAX_USDC_LP",
            "PNG",
            "PNG_AVAX_USDC_LP",
            "QI",
            "SAV2",
            "TJ_AVAX_USDC_LP",
            "USDC",
            "USDT",
            "XAVA",
            "YAK",
            "YYAV3SA1",
            "YY_TJ_AVAX_USDC_LP",
            "sAVAX",
        ].to_vec()).await;
        assert_ne!(result, "");
    }

    #[tokio::test]
    async fn it_works_for_two_assets() {
        let result = get_prices("".parse().unwrap(), ["AVAX", "ETH"].to_vec(), "redstone-avalanche-prod-1".to_string(), [
            "AVAX",
            "BTC",
            "ETH",
            "FRAX",
            "LINK",
            "MOO_TJ_AVAX_USDC_LP",
            "PNG",
            "PNG_AVAX_USDC_LP",
            "QI",
            "SAV2",
            "TJ_AVAX_USDC_LP",
            "USDC",
            "USDT",
            "XAVA",
            "YAK",
            "YYAV3SA1",
            "YY_TJ_AVAX_USDC_LP",
            "sAVAX",
        ].to_vec()).await;
        println!("{:?}", result);
        assert_ne!(result, "");
    }

    #[tokio::test]
    async fn it_works_for_ten_assets() {
        let result = get_prices("".parse().unwrap(), [
            "AVAX",
            "ETH",
            "BTC",
            "USDT",
            "PNG",
            "XAVA",
            "LINK",
            "YAK",
            "QI",
            "USDC",
        ].to_vec(), "redstone-avalanche-prod-1".to_string(),
                                [
                                    "AVAX",
                                    "BTC",
                                    "ETH",
                                    "FRAX",
                                    "LINK",
                                    "MOO_TJ_AVAX_USDC_LP",
                                    "PNG",
                                    "PNG_AVAX_USDC_LP",
                                    "QI",
                                    "SAV2",
                                    "TJ_AVAX_USDC_LP",
                                    "USDC",
                                    "USDT",
                                    "XAVA",
                                    "YAK",
                                    "YYAV3SA1",
                                    "YY_TJ_AVAX_USDC_LP",
                                    "sAVAX",
                                ].to_vec()).await;
        println!("{:?}", result);
        assert_ne!(result, "");
    }

    #[tokio::test]
    async fn it_works_for_a_package() {
        let result = get_packages(
            "".parse().unwrap(),
            3,
            [
                "AVAX".to_string(),
                "BTC".to_string(),
                "BUSD".to_string(),
                "ETH".to_string(),
                "GLP".to_string(),
                "GMX".to_string(),
                "JOE".to_string(),
                "LINK".to_string(),
                "MOO_TJ_AVAX_USDC_LP".to_string(),
                "PNG".to_string(),
                "PNG_AVAX_ETH_LP".to_string(),
                "PNG_AVAX_USDC_LP".to_string(),
                "PNG_AVAX_USDT_LP".to_string(),
                "PTP".to_string(),
                "QI".to_string(),
                "TJ_AVAX_BTC_LP".to_string(),
                "TJ_AVAX_ETH_LP".to_string(),
                "TJ_AVAX_USDC_LP".to_string(),
                "TJ_AVAX_USDT_LP".to_string(),
                "TJ_AVAX_sAVAX_LP".to_string(),
                "USDC".to_string(),
                "USDT".to_string(),
                "XAVA".to_string(),
                "YAK".to_string(),
                "YYAV3SA1".to_string(),
                "YY_AAVE_AVAX".to_string(),
                "YY_GLP".to_string(),
                "YY_PNG_AVAX_ETH_LP".to_string(),
                "YY_PNG_AVAX_USDC_LP".to_string(),
                "YY_PTP_sAVAX".to_string(),
                "YY_TJ_AVAX_ETH_LP".to_string(),
                "YY_TJ_AVAX_USDC_LP".to_string(),
                "YY_TJ_AVAX_sAVAX_LP".to_string(),
                "sAVAX".to_string(),
            ].to_vec(),
            ["___ALL_FEEDS___".to_string()].to_vec(),
        ).await;
        println!("{:?}", result);
        assert_ne!(result, "");
    }
}
