//! Will provide means to use Redstone in Rust
//!
//! Will provides functions to interact with Redstone's
//! [`Redstone`]: https://redstone.finance/

use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use ethers::abi::AbiEncode;
use ethers::utils::__serde_json::to_vec;
use redstone_api::{get_package, get_price};

/// Function that will add at the end of the data the redstone specific data that we will craft
/// It returns the data it got as input + extra, where extra is generated following redstone logic
pub async fn get_prices(data: String, vec_assets: Vec<String>, provider: String) -> String {
    let mut assets: Option<String> = Some(String::new());
    if vec_assets.is_empty() {
        assets = None;
    }
    let vec_len = vec_assets.len();
    for asset in vec_assets {
        let mut assetss = assets.unwrap();
        assetss += asset.as_str();
        if vec_len > 1 {
            assetss += ",";
        }

        assets = Some(assetss);
    }

    //ToDo Rename this
    let vec_response_api = get_price("https://api.redstone.finance/prices?{symbol}={assets}&provider={provider}".parse().unwrap(), assets, provider).await;

    let mut serialized_data = SerializedPriceData {
        map_symbol_value: HashMap::new(),
        timestamp: 0,
        lite_sig: String::new(),
    };

    serialized_data.timestamp = vec_response_api.get(0).unwrap().timestamp.unwrap();
    serialized_data.lite_sig = vec_response_api.get(0).unwrap().lite_evm_signature.clone().unwrap();
    for r in vec_response_api {
        serialized_data.map_symbol_value.insert(r.symbol.unwrap(), (r.value.unwrap() * 100000000.).round() as u64);
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
pub async fn get_packages(data: String, provider: String) -> String {
    //ToDo Rename this
    let vec_response_api = get_package("https://api.redstone.finance/packages/latest?provider={provider}".parse().unwrap(), provider).await;

    let mut serialized_data = SerializedPriceData {
        map_symbol_value: HashMap::new(),
        timestamp: 0,
        lite_sig: String::new(),
    };

    serialized_data.timestamp = vec_response_api.timestamp.unwrap();
    serialized_data.lite_sig = vec_response_api.lite_signature.clone().unwrap();
    for r in vec_response_api.prices {
        serialized_data.map_symbol_value.insert(r.symbol.unwrap(), (r.value.unwrap() * 100000000.).round() as u64);
        // serialized_data.symbols.push(r.symbol.unwrap());
        // serialized_data.values.push((r.value.unwrap() as u128 * 100000000.).round() as u64);
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

pub fn get_lite_data_bytes_string(price_data: SerializedPriceData) -> String {
    let mut data = String::new();

    let data_order = [
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
    ].to_vec();


    for (_, symbol) in data_order.into_iter().enumerate() {
        let symbol = symbol;
        let value = price_data.map_symbol_value.get(symbol).unwrap();
        let b32 = ethers::utils::format_bytes32_string(&*symbol).unwrap();
        let b32_hex = b32.encode_hex();
        let b32_hex_stripped = b32_hex.strip_prefix("0x").unwrap();
        data += b32_hex_stripped;
        data += value.encode_hex().strip_prefix("0x").unwrap();
    }
    let timestamp = (price_data.timestamp as f64 / 1000.).ceil() as u64;
    let timestamp_hex = timestamp.encode_hex();
    let timestamp_hex_stripped = timestamp_hex.strip_prefix("0x").unwrap();

    data += timestamp_hex_stripped;

    let len_hex = format!("{:#04x}", price_data.map_symbol_value.len());
    let len_hex = len_hex.strip_prefix("0x").unwrap();

    data += len_hex;

    let lite_sig = price_data.lite_sig.clone();
    let lite_sig = lite_sig.strip_prefix("0x").unwrap();

    data += lite_sig;


    data
}

#[derive(Debug)]
pub struct SerializedPriceData {
    map_symbol_value: HashMap<String,u64>,
    timestamp: u64,
    lite_sig: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works_for_one_asset() {
        let result = get_prices("".parse().unwrap(), ["AVAX".to_string()].to_vec(), "redstone-avalanche-prod-1".to_string()).await;
        assert_ne!(result, "");
    }

    #[tokio::test]
    async fn it_works_for_two_assets() {
        let result = get_prices("".parse().unwrap(), ["AVAX".to_string(), "ETH".to_string()].to_vec(), "redstone-avalanche-prod-1".to_string()).await;
        println!("{:?}", result);
        assert_ne!(result, "");
    }

    #[tokio::test]
    async fn it_works_for_ten_assets() {
        let result = get_prices("".parse().unwrap(), [
            "AVAX".to_string(),
            "ETH".to_string(),
            "BTC".to_string(),
            "USDT".to_string(),
            "PNG".to_string(),
            "XAVA".to_string(),
            "LINK".to_string(),
            "YAK".to_string(),
            "QI".to_string(),
            "USDC".to_string(),
        ].to_vec(), "redstone-avalanche-prod-1".to_string()).await;
        println!("{:?}", result);
        assert_ne!(result, "");
    }

    #[tokio::test]
    async fn it_works_for_a_package() {
        let result = get_packages("".parse().unwrap(), "redstone-avalanche-prod-1".to_string()).await;
        println!("{:?}", result);
        assert_ne!(result, "");
    }
}
