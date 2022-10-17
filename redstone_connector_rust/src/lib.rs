use ethers::abi::AbiEncode;
use redstone_api::call;

//TODO Find good name
/// Function that will add at the end of the data the redstone specific data that we will craft
/// It returns the data it got as input + extra
pub async fn add_redstone_data(data: String, assets: Vec<String>) -> String {
    // It needs to call a redstone endpoint with appropriate request param
    let res = call("https://api.redstone.finance/prices?symbol=AVAX&provider=redstone-avalanche-prod-1&limit=1".parse().unwrap(), Vec::new()).await;
    // Deserialize the response
    let mut serialized_data = SerializedPriceData {
        symbols: vec![],
        values: vec![],
        timestamp: 0,
        lite_sig: String::new(),
    };

    serialized_data.timestamp = res.get(0).unwrap().timestamp.unwrap();
    serialized_data.symbols.push(res.get(0).unwrap().symbol.clone().unwrap());
    let vv = (res.get(0).unwrap().value.unwrap() * 100000000.) as u64;
    // let vv = 1603300000;
    serialized_data.values.push(vv);
    serialized_data.lite_sig = res.get(0).unwrap().lite_evm_signature.clone().unwrap();
    // call get_lite_data_bytes_string
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
        // let value = 1565078250;
        // let value = 1603000000;
        let b32 = ethers::utils::format_bytes32_string(&*symbol).unwrap();
        let b32_hex = b32.encode_hex();
        let b32_hex_stripped = b32_hex.strip_prefix("0x").unwrap();
        data += b32_hex_stripped;
        data += value.encode_hex().strip_prefix("0x").unwrap();

        let timestamp = (price_data.timestamp as f64 / 1000.).ceil() as u64;
        // let timestamp = 1665487114642_u64 / 1000;
        let timestamp_hex = timestamp.encode_hex();
        let timestamp_hex_stripped = timestamp_hex.strip_prefix("0x").unwrap();

        data += timestamp_hex_stripped;

        let len_hex = format!("{:#04x}", price_data.values.len());
        let len_hex = len_hex.strip_prefix("0x").unwrap();

        data += len_hex;

        let lite_sig = price_data.lite_sig.clone();
        let lite_sig = lite_sig.strip_prefix("0x").unwrap();

        data += lite_sig;

        println!(
            "OYYYYYYH {:02X?}",
            ethers::utils::format_bytes32_string(&*symbol).unwrap().encode_hex().strip_prefix("0x")
        );
        println!("OYYYYYYH - 2 {:?}", value.encode_hex().strip_prefix("0x"));
        println!("OYYYYYYH - 2 {:?}", data);
    }

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
    async fn it_works() {
        let result = add_redstone_data("".parse().unwrap(), Vec::new()).await;
        assert_ne!(result, "");
    }
}
