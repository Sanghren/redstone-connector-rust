//! Will provide means to use Redstone in Rust
//!
//! Will provides functions to interact with Redstone's
//! [`Redstone`]: https://redstone.finance/

use ethers::abi::AbiEncode;
use redstone_api::{get_price};

/// Function that will add at the end of the data the redstone specific data that we will craft
/// It returns the data it got as input + extra, where extra is generated following redstone logic
pub async fn add_redstone_data(data: String, vec_assets: Vec<String>) -> String {
    let mut assets = String::new();
    let vec_len = vec_assets.len();
    for asset in vec_assets {
        assets += asset.as_str();
        if vec_len > 1 {
            assets += ",";
        }
    }

    //ToDo Rename this
    let vec_response_api = get_price("https://api.redstone.finance/prices?{symbol}={assets}&provider=redstone-avalanche-prod-1&limit=1".parse().unwrap(), assets).await;

    let mut serialized_data = SerializedPriceData {
        symbols: vec![],
        values: vec![],
        timestamp: 0,
        lite_sig: String::new(),
    };

    serialized_data.timestamp = vec_response_api.get(0).unwrap().timestamp.unwrap();
    serialized_data.lite_sig = vec_response_api.get(0).unwrap().lite_evm_signature.clone().unwrap();
    for r in vec_response_api {
        serialized_data.symbols.push(r.symbol.unwrap());
        serialized_data.values.push((r.value.unwrap() * 100000000.) as u64);
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

    for (index, symbol) in price_data.symbols.into_iter().enumerate() {
        let symbol = symbol;
        let value = price_data.values.get(index).unwrap();
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

    let len_hex = format!("{:#04x}", price_data.values.len());
    let len_hex = len_hex.strip_prefix("0x").unwrap();

    data += len_hex;

    let lite_sig = price_data.lite_sig.clone();
    let lite_sig = lite_sig.strip_prefix("0x").unwrap();

    data += lite_sig;


    data
}

pub struct SerializedPriceData {
    symbols: Vec<String>,
    values: Vec<u64>,
    timestamp: u64,
    lite_sig: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works_for_one_asset() {
        let result = add_redstone_data("".parse().unwrap(), ["AVAX".to_string()].to_vec()).await;
        assert_ne!(result, "");
    }

    #[tokio::test]
    async fn it_works_for_two_assets() {
        let result = add_redstone_data("".parse().unwrap(), ["AVAX".to_string(), "ETH".to_string()].to_vec()).await;
        println!("{:?}", result);
        assert_ne!(result, "");
    }

    #[tokio::test]
    async fn it_works_for_ten_assets() {
        let result = add_redstone_data("".parse().unwrap(), [
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
        ].to_vec()).await;
        println!("{:?}", result);
        assert_ne!(result, "");
    }
}
