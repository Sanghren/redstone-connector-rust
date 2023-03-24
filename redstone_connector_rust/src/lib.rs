//! Will provide means to use Redstone in Rust
//!
//! Will provides functions to interact with Redstone's
//! [`Redstone`]: https://redstone.finance/
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fmt::Debug;
use std::io::BufRead;
use std::ops::Mul;
use std::str::FromStr;
use std::time::Duration;
use ethers::abi::AbiEncode;
use ethers::utils::__serde_json::to_vec;
use ethers::utils::{format_bytes32_string, hex};
use log::{debug, error, info, trace};
use redstone_api::{get_package, get_price};
use data_encoding::BASE64;
use data_encoding::HEXLOWER;
use ethers::utils::hex::ToHex;
use ethers_solc::resolver::print;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, One, ToPrimitive};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use ethers::types::BlockId::Hash;
use ethers::utils::rlp::Prototype::Data;

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
        let fixed_decimal_num = r.value.unwrap();
        println!("{}", fixed_decimal_num);
        serialized_data.map_symbol_value.insert(r.symbol.unwrap(), fixed_decimal_num);        // serialized_data.symbols.push(r.symbol.unwrap());
        // serialized_data.values.push((r.value.unwrap() * 100000000.).round() as u64);
    }

    // ToDo It must work for an array with more than 1 asset
    // serialized_data.symbols.push(vec_response_api.get(0).unwrap().symbol.clone().unwrap());
    // let value = (vec_response_api.get(0).unwrap().value.unwrap() * 100000000.) as u64;
    // serialized_data.values.push(value);
    let data_to_append = get_lite_data_bytes_string(serialized_data, 35_usize);

    // append the result of the above line to input data
    let new_data = data + &*data_to_append;
    // return the whole things
    new_data
}

/// Function that will add at the end of the data the redstone specific data that we will craft
/// It returns the data it got as input + extra, where extra is generated following redstone logic
pub async fn get_packages(base_call_data_vec: Vec<String>, number_of_data_package: usize, order_of_assets: Vec<String>, data_feeds: Vec<String>) -> Vec<String> {
    let data_feeds_ids = if data_feeds.is_empty() { ["___ALL_FEEDS___".to_string()].to_vec() } else { data_feeds };

    //ToDo Rename this
    let map_response_api = get_package("https://oracle-gateway-2.a.redstone.finance/data-packages/latest/redstone-avalanche-prod".parse().unwrap()).await;
    let mut redstone_call_data = Vec::new();

    for base_call_data in base_call_data_vec {
        let mut new_data = base_call_data;
        let mut i = 0_usize;
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

                // println!("Key {}", asset);
                for data_point in &map_response_api.get("___ALL_FEEDS___").unwrap().get(i).unwrap().dataPoints {
                    if asset.eq_ignore_ascii_case(&data_point.dataFeedId) {
                        let fixed_decimal_num = data_point.value;
                        // println!("{} // {}", data_point.value, fixed_decimal_num);
                        serialized_data.map_symbol_value.insert(asset.clone(), fixed_decimal_num);
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
            let data_to_append = get_lite_data_bytes_string(serialized_data, order_of_assets.len());
            new_data += &*data_to_append;
            i += 1;
        }
        // append the result of the above line to input data

        add_meta_data_bytes(&mut new_data);
        redstone_call_data.push(new_data.clone());
    }


    // return the whole things
    redstone_call_data
}

pub fn get_lite_data_bytes_string(price_data: SerializedPriceData, number_of_data_points: usize) -> String {
    let mut data = String::new();
    let len_map = price_data.map_symbol_value.len();
    for (symbol, value) in price_data.map_symbol_value.iter() {
        let symbol = symbol;
        trace!("Processing information about {:?}", symbol);
        let b32 = ethers::utils::format_bytes32_string(&*symbol).unwrap();
        let b32_hex = b32.encode_hex();
        let b32_hex_stripped = b32_hex.strip_prefix("0x").unwrap();
        data += b32_hex_stripped;

        // let num = Decimal::from_str(110462390.9453476.to_string().as_str()).unwrap();
        ; // 6a10d884
        println!("RAW {} // STRING {}", symbol, value.to_string().as_str());
        let hex_string = generate_price_data(value);
        // Prints "6a10d884"

        // println!("hex_string : {}", hex_string);
        data += hex_string.as_str();
    }
    let timestamp = price_data.timestamp as u64;
    // let tmstmp = Duration::from_secs(timestamp);
    // println!("Timestamp : {:?}", timestamp);
    let timestamp_hex = format!("{:#04x}", timestamp);
    // println!("Timestamp : {:?}", timestamp_hex);
    let timestamp_hex_stripped = timestamp_hex.strip_prefix("0x").unwrap();
    // println!("Timestamp : {:?}", timestamp_hex_stripped);
    data += "0";
    data += timestamp_hex_stripped;

    let data_point_size_hex = format!("{:0>8x}", 32);
    // println!("data_point_size_hex : {:?}", data_point_size_hex);

    data += data_point_size_hex.as_str();

    // ToDo Automatic and not hardcoded
    let data_point_number_hex = format!("{:0>6x}", number_of_data_points);
    // println!("data_point_number_hex : {:?}", data_point_number_hex);

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

fn generate_price_data(value: &f64) -> String {
    let num = Decimal::from_str(value.to_string().as_str()).unwrap();
    // If 9th decimal is 5 then ...
    let mut scaled_num = 0_u128;
    let res = get_decimal_place(9, num.clone());
    println!("Final number 9 in decimal is {}", res);
    if res > 5 {
        scaled_num = (((num * Decimal::from_f64(100000000_f64).unwrap()).ceil() / Decimal::from_f64(100000000_f64).unwrap()) * Decimal::from_f64(100000000_f64).unwrap()).round().to_u128().unwrap();
    } else if res == 0 {
        let res = get_decimal_place(8, num.clone());
        println!("Final number 8 in decimal is {}", res);
        if res == 1 {
            scaled_num = (((num * Decimal::from_f64(100000000_f64).unwrap()).floor() / Decimal::from_f64(100000000_f64).unwrap()) * Decimal::from_f64(100000000_f64).unwrap()).to_u128().unwrap();
        } else if res == 0 {
            let res = get_decimal_place(7, num.clone());
            println!("Final number 7 in decimal is {}", res);
            if res >= 5 {
                scaled_num = (((num * Decimal::from_f64(100000000_f64).unwrap() + Decimal::one()).floor() / Decimal::from_f64(100000000_f64).unwrap()) * Decimal::from_f64(100000000_f64).unwrap()).to_u128().unwrap();
            } else {
                scaled_num = (((num * Decimal::from_f64(100000000_f64).unwrap()).floor() / Decimal::from_f64(100000000_f64).unwrap()) * Decimal::from_f64(100000000_f64).unwrap()).to_u128().unwrap();
            }
        } else {
            scaled_num = (((num * Decimal::from_f64(100000000_f64).unwrap()).floor() / Decimal::from_f64(100000000_f64).unwrap()) * Decimal::from_f64(100000000_f64).unwrap()).to_u128().unwrap();
        }
    } else {
        scaled_num = (((num * Decimal::from_f64(100000000_f64).unwrap()).floor() / Decimal::from_f64(100000000_f64).unwrap()) * Decimal::from_f64(100000000_f64).unwrap()).to_u128().unwrap();
    }
    // let scaled_num = (num * 100000000_f64.round()) as u64;
    // let scaled_num = scaled_num as f64;
    // let scaled_num = scaled_num / 100000000_f64;
    // let scaled_num = (scaled_num * 100000000_f64).round();
    // let scaled_num = scaled_num as u64;
    // let big_deci = Decimal::from_str("133018818.04845291").unwrap();
    // let scaled_num = (((num * 100000000_f64).round() as u64 as f64 / 100000000_f64) * 100000000_f64) as u64;
    // println!("scaled_num 1 {}", scaled_num);
    // println!("big_deci 1 {}", big_deci);
    // println!("scaled_num 2 {}", big_deci * BigDecimal::from_f64om_f64(100000000_f64).unwrap());
    // let scaled_num_with_prec = (big_deci * Decimal::from_f64(100000000_f64).unwrap());
    // println!("scaled_num 2a {}", scaled_num_with_prec);
    // println!("scaled_num 3 {}", (num * Decimal::from_f64(100000000_f64).unwrap()).floor());
    // println!("scaled_num 4 {}", ((num * Decimal::from_f64(100000000_f64).unwrap()).floor() / Decimal::from_f64(100000000_f64).unwrap()));
    // println!("scaled_num 4 {}", (((num * Decimal::from_f64(100000000_f64).unwrap()).floor() / Decimal::from_f64(100000000_f64).unwrap()) * Decimal::from_f64(100000000_f64).unwrap()).round().to_u128().unwrap());
    let bytes = scaled_num.to_be_bytes();
    let hex_string = format!("{:0>64}", hex::encode(bytes));

    println!("qqqqq {}", hex_string); // Prints "6a10d884"
    hex_string
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

fn bytes_arr_to_number(number_bytes: &[u8]) -> u128 {
    let mut result_number = 0;
    let mut multiplier = 1;

    for i in (0..number_bytes.len()).rev() {
        // To prevent overflow error
        if i == 16 {
            break;
        }
        result_number += u128::from(number_bytes[i]) * multiplier;
        multiplier *= 256;
    }

    result_number
}

fn get_decimal_place(x: u32, num: Decimal) -> u64 {

    // var result = value / (int)Math.Pow(10, position);
    // result = result % 10;
    // return result;
    // println!("AAA {}", num);

    let shifted = num.mul(Decimal::from_f64(10_f64.powi(x as i32)).unwrap());
    // println!("AAA {}", shifted);
    // println!("AAA {}", shifted.to_f64().unwrap());
    let truncated = shifted % Decimal::from_f64(10.0).unwrap();
    // println!("AAA {}", truncated);
    truncated.to_u64().unwrap()
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

const REDSTONE_MARKER_BS: usize = 18;
const UNSIGNED_METADATA_BYTE_SIZE_BS: usize = 6;
const DATA_PACKAGES_COUNT_BS: usize = 4;
const DATA_POINTS_COUNT_BS: usize = 6;
const SIGNATURE_BS: usize = 130;
const MAX_SIGNERS_COUNT: usize = 512;
const DATA_POINT_VALUE_BYTE_SIZE_BS: usize = 8;
const DATA_FEED_ID_BS: usize = 64;
const TIMESTAMP_BS: usize = 12;

// ========================== RedStone payload structure (hex) ==========================
//   "4254430000000000000000000000000000000000000000000000000000000000" + // bytes32("BTC")
//   "000000000000000000000000000000000000000000000000000003d1e3821000" + // 42000 * 10^8
//   "4554480000000000000000000000000000000000000000000000000000000000" + // bytes32("ETH")
//   "0000000000000000000000000000000000000000000000000000002e90edd000" + // 2000 * 10^8
//   "01812f2590c0" + // timestamp (1654353400000 in hex)
//   "00000020" + // data points value byte size (32 in hex)
//   "000002" + // data points count
//   "c1296a449f5d353c8b04eb389f33a583ee79449cca6e366900042f19f2521e722a410929223231905839c00865af68738f1a202478d87dc33675ea5824f343901b" + // signature of the first signer
//   "4254430000000000000000000000000000000000000000000000000000000000" + // bytes32("BTC")
//   "000000000000000000000000000000000000000000000000000003d1e3821000" + // 42000 * 10^8
//   "4554480000000000000000000000000000000000000000000000000000000000" + // bytes32("ETH")
//   "0000000000000000000000000000000000000000000000000000002e90edd000" + // 2000 * 10^8
//   "01812f2590c0" + // timestamp (1654353400000 in hex)
//   "00000020" + // data points value byte size (32 in hex)
//   "000002" + // data points count
//   "dbbf8a0e6b1c9a56a4a0ef7089ef2a3f74fbd21fbd5c7c8192b70084004b4f6d37427507c4fff835f74fd4d000b6830ed296e207f49831b96f90a4f4e60820ee1c" + // signature of the second signer
//   "0002" + // data packages count
//   "312e312e3223746573742d646174612d66656564" + // unsigned metadata toUtf8Bytes("1.1.2#test-data-feed")
//   "000014" + // unsigned metadata byte size (20 in hex)
//   "000002ed57011e0000" // RedStone marker

fn compare_data_extraction_results(
    a: &DataExtractionResult,
    b: &DataExtractionResult,
) -> String {
    let mut report = String::new();

    if a.redstone_marker != b.redstone_marker {
        report.push_str(&format!(
            "Redstone marker: {} | {}\n",
            a.redstone_marker, b.redstone_marker
        ));
    }
    if a.unsigned_metadata_byte_size != b.unsigned_metadata_byte_size {
        report.push_str(&format!(
            "Unsigned metadata byte size: {} | {}\n",
            a.unsigned_metadata_byte_size, b.unsigned_metadata_byte_size
        ));
    }
    if a.unsigned_metadata != b.unsigned_metadata {
        report.push_str(&format!(
            "Unsigned metadata: {} | {}\n",
            a.unsigned_metadata, b.unsigned_metadata
        ));
    }
    if a.data_packages_count != b.data_packages_count {
        report.push_str(&format!(
            "Data packages count: {} | {}\n",
            a.data_packages_count, b.data_packages_count
        ));
    }

    for dp_a in a.data_packages.iter() {
        let mut dp_b = b.data_packages.iter().find(|&dp_b| {
            println!("DP A {} // MI B {}", dp_a.signature, dp_b.signature);
            dp_a.signature == dp_b.signature
        });

        if dp_b.is_none() {
            continue;
        }
        let dp_b = dp_b.unwrap();
        if dp_a.signature != dp_b.signature {
            report.push_str(&format!(
                "  Signature: DP {} | MI {}\n",
                dp_a.signature, dp_b.signature
            ));
        } else {
            println!("DP Sig A {} // MI Sig B {}", dp_a.signature, dp_b.signature);
            if dp_a.data_points_count != dp_b.data_points_count {
                report.push_str(&format!(
                    "  Data points count: {} | {}\n",
                    dp_a.data_points_count, dp_b.data_points_count
                ));
            }
            if dp_a.data_point_size != dp_b.data_point_size {
                report.push_str(&format!(
                    "  Data point size: {} | {}\n",
                    dp_a.data_point_size, dp_b.data_point_size
                ));
            }
            if dp_a.timestamp != dp_b.timestamp {
                report.push_str(&format!("  Timestamp: {} | {}\n", dp_a.timestamp, dp_b.timestamp));
            }

            for (j, (point_a, point_b)) in dp_a.data_points.iter().zip(dp_b.data_points.iter()).enumerate() {
                if point_a.value != point_b.value {
                    report.push_str(&format!("    DataPoint #{}: DP {} | MI {}\n", j + 1, point_a.value, point_b.value));
                    report.push_str(&format!("    DataPoint #{}: DP {} | MI {}\n", j + 1, point_a.token, point_b.token));
                }
                if point_a.token != point_b.token {
                    report.push_str(&format!("    DataPoint #{}: DP {} | MI {}\n", j + 1, point_a.value, point_b.value));
                    report.push_str(&format!("    DataPoint #{}: DP {} | MI {}\n", j + 1, point_a.token, point_b.token));
                }
            }
        }
    }

    report
}


#[derive(Debug)]
pub struct DataExtractionResult {
    pub redstone_marker: String,
    pub unsigned_metadata_byte_size: u32,
    pub unsigned_metadata: String,
    pub data_packages_count: u32,
    pub data_packages: Vec<DataPackage>,
}

#[derive(Debug)]
pub struct DataPackage {
    pub signature: String,
    pub data_points_count: u32,
    pub data_point_size: u32,
    pub timestamp: u128,
    pub data_packages_count: u32,
    pub data_points: Vec<DataPoint>,
}

#[derive(Debug)]
pub struct DataPoint {
    pub value: u128,
    pub token: String,
}

impl Default for DataExtractionResult {
    fn default() -> Self {
        DataExtractionResult {
            redstone_marker: "".to_string(),
            unsigned_metadata_byte_size: 0,
            unsigned_metadata: "".to_string(),
            data_packages_count: 0,
            data_packages: vec![],
        }
    }
}

impl Default for DataPackage {
    fn default() -> Self {
        DataPackage {
            signature: "".to_string(),
            data_points_count: 0,
            data_point_size: 0,
            timestamp: 0,
            data_packages_count: 0,
            data_points: vec![],
        }
    }
}

impl Default for DataPoint {
    fn default() -> Self {
        DataPoint {
            value: 0,
            token: "".to_string(),
        }
    }
}

fn compare_delta_prime_call_data_to_mine(mut call_data: &mut [u8]) -> DataExtractionResult {
    // Have to start from the end
    let mut extracted_data = DataExtractionResult::default();
    let mut cursor = call_data.len();
    println!("cursor {}", cursor);

    // START -- EXTRACT AND TODO CHECK THE REDSTONE MARKER
    let mut call_data_redstone_marker: Vec<u8> = Vec::new();
    (call_data, call_data_redstone_marker, cursor) = split_call_data(call_data, cursor, REDSTONE_MARKER_BS);
    // println!("DP REDSTONE_MARKER_BS {:?}", call_data_redstone_marker);
    extracted_data.redstone_marker = format!("{:?}", call_data_redstone_marker);
    // // END -- EXTRACT AND TODO CHECK THE REDSTONE MARKER

    // START -- EXTRACT UNSIGNED_METADATA_BYTE_SIZE_BS
    let mut call_data_unsigned_metadata_byte_size: Vec<u8> = Vec::new();
    (call_data, call_data_unsigned_metadata_byte_size, cursor) = split_call_data(call_data, cursor, UNSIGNED_METADATA_BYTE_SIZE_BS);
    // println!("REDSTONE_MARKER_BS {:?}", call_data_unsigned_metadata_byte_size);
    // Convert u8 array to string
    let s = String::from_utf8_lossy(&call_data_unsigned_metadata_byte_size).to_string();
    // Parse string as integer with base 16
    let call_data_unsigned_metadata_size = u32::from_str_radix(&s, 16).unwrap();
    extracted_data.unsigned_metadata_byte_size = call_data_unsigned_metadata_size;

    // println!("UNSIGNED_METADATA_BYTE_SIZE_BS {:?}", call_data_unsigned_metadata_size);
    // END -- EXTRACT UNSIGNED_METADATA_BYTE_SIZE_BS

    // START -- EXTRACT UNSIGNED_METADATA
    let mut call_data_unsigned_metadata: Vec<u8> = Vec::new();
    (call_data, call_data_unsigned_metadata, cursor) = split_call_data(call_data, cursor, (call_data_unsigned_metadata_size * 2) as usize);
    extracted_data.unsigned_metadata = format!("{:?}", call_data_unsigned_metadata);
    // println!("UNSIGNED_METADATA {:?}", call_data_unsigned_metadata);
    // END -- EXTRACT UNSIGNED_METADATA

    // START -- EXTRACT DATA_PACKAGES_COUNT_BS
    let mut call_data_unsigned_metadata: Vec<u8> = Vec::new();
    (call_data, call_data_unsigned_metadata, cursor) = split_call_data(call_data, cursor, DATA_PACKAGES_COUNT_BS);

    // Convert u8 array to string
    let s = String::from_utf8_lossy(&call_data_unsigned_metadata).to_string();
    // Parse string as integer with base 16
    let call_data_packages_count = u8::from_str_radix(&s, 16).unwrap();
    println!("cursor B {}", cursor);

    // println!("DATA_PACKAGES_COUNT_BS {:?}", call_data_packages_count);
    // END -- EXTRACT DATA_PACKAGES_COUNT_BS
    // START -- LOOP EXTRACT MAIN DATA COMPONENT
    for count in 0..call_data_packages_count {
        // START -- IN LOOP EXTRACT SIGNATURE
        println!("cursor 1111 A {} {} {}", call_data.len(), cursor, count);

        let mut data_package = DataPackage::default();
        let mut call_data_signature: Vec<u8> = Vec::new();
        (call_data, call_data_signature, cursor) = split_call_data(call_data, cursor, SIGNATURE_BS);

        println!("cursor 1111 B {} {} {}", call_data.len(), cursor, count);
        data_package.signature = String::from_utf8_lossy(call_data_signature.as_slice()).to_string();
        // END -- IN LOOP EXTRACT SIGNATURE

        // START -- EXTRACT DATA_PACKAGES_COUNT_BS

        let mut call_data_data_points_count: Vec<u8> = Vec::new();
        (call_data, call_data_data_points_count, cursor) = split_call_data(call_data, cursor, DATA_POINTS_COUNT_BS);
        println!("cursor 1111 C {} {}", call_data.len(), cursor);

        // Convert u8 array to string
        let s = String::from_utf8_lossy(&call_data_data_points_count.as_slice()).to_string();
        // Parse string as integer with base 16
        let call_data_points_count = u32::from_str_radix(&s, 16).unwrap();

        println!("DATA_PACKAGES_COUNT_BS {:?}", call_data_points_count);
        // END -- EXTRACT DATA_PACKAGES_COUNT_BS

        // START -- EXTRACT DATA_POINT_VALUE_BYTE_SIZE_BS
        let mut call_data_unsigned_metadata_byte_size: Vec<u8> = Vec::new();
        (call_data, call_data_unsigned_metadata_byte_size, cursor) = split_call_data(call_data, cursor, DATA_POINT_VALUE_BYTE_SIZE_BS);


        // Convert u8 array to string
        let s = String::from_utf8_lossy(&call_data_unsigned_metadata_byte_size).to_string();
        // Parse string as integer with base 16
        let call_data_unsigned_metadata_size = u32::from_str_radix(&s, 16).unwrap();
        data_package.data_point_size = call_data_unsigned_metadata_size;

        println!("DP UNSIGNED_DATA_POINT_BYTE_SIZE_BS {:?}", call_data_unsigned_metadata_size);
        // END -- EXTRACT DATA_POINT_VALUE_BYTE_SIZE_BS

        // START -- EXTRACT TIMESTAMP_BS
        let mut call_data_timestamp: Vec<u8> = Vec::new();
        (call_data, call_data_timestamp, cursor) = split_call_data(call_data, cursor, TIMESTAMP_BS);

        // println!("DP CALL DATA TIMESTAMP AFTER {:?} -- TMPSTPM END", call_data_timestamp);

        // Convert u8 array to string
        let s = String::from_utf8_lossy(&call_data_timestamp.as_slice()).to_string();
        // Parse string as integer with base 16
        let call_data_timestamp = u128::from_str_radix(&s, 16).unwrap();
        data_package.timestamp = call_data_timestamp;

        println!("DP TIMESTAMP_BS {:?}", call_data_timestamp);
        // END -- EXTRACT TIMESTAMP_BS

        for count in 0..call_data_points_count {
            let mut data_point = DataPoint::default();
            // println!("DP Data points count : {} // {}", count, cursor);
            let mut token = "";
            let mut value = 0_u128;
            // (call_data, _, cursor) = split_call_data(call_data, cursor, (count as usize * 128_usize));

            // START -- EXTRACT DATAPOINT VALUE
            let mut call_data_point_value: Vec<u8> = Vec::new();
            (call_data, call_data_point_value, cursor) = split_call_data(call_data, cursor, 64_usize);
            println!("DP CALL DATA DATAPOINT VALUE AFTER {:?} -- DATAPOINT VALUE END", call_data_point_value);

            // Convert u8 array to string
            let s = String::from_utf8_lossy(&call_data_point_value.as_slice()).to_string();
            // Parse string as integer with base 16
            let call_data_point_value = u128::from_str_radix(&s, 16).unwrap();
            data_point.value = call_data_point_value;
            // println!("DP DATA POINT VALUE {:?}", call_data_point_value);
            // END -- EXTRACT DATAPOINT VALUE

            // START -- EXTRACT DATAPOINT TOKEN
            let mut call_data_point_token: Vec<u8> = Vec::new();
            (call_data, call_data_point_token, cursor) = split_call_data(call_data, cursor, 64_usize);

            // println!("DP CALL DATA POINT TOKEN {:?} -- POINT TOKEN END", call_data_point_token);
            data_point.token = String::from_utf8_lossy(call_data_point_token.as_slice()).to_string();


            // println!("DP DATA POINT TOKEN {:?}", String::from_utf8_lossy(call_data_point_token));
            // END -- EXTRACT DATAPOINT TOKEN
            data_package.data_points.push(data_point);
        }
        extracted_data.data_packages.push(data_package);
    }

    extracted_data
    // END -- LOOP EXTRACT MAIN DATA COMPONENT
}

fn split_call_data<'a>(mut call_data: &mut [u8], mut dp_cursor: usize, position_split: usize) -> (&mut [u8], Vec<u8>, usize) {
    // println!("In split call data 1 {}", call_data.len());
    dp_cursor = call_data.len() - (position_split);

    let mut call_data_redstone_marker: &mut [u8] = &mut [];
    (call_data, call_data_redstone_marker) = call_data.split_at_mut((dp_cursor).into());
    // println!("In split call data 2 {}", call_data.len());
    (call_data, call_data_redstone_marker.to_vec(), dp_cursor)
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
    async fn test_01() {
        let result = generate_price_data(
            &10035167.405031875
        );
        println!("{:?}", result);
        assert_eq!(result.to_ascii_lowercase(), format!("{:0>64}", "390B172D80693").to_ascii_lowercase());
    }

    #[tokio::test]
    async fn test_call_data() {
        let mut dp = "0xd44e282b41564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000067c89a804254430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002933d6e477c42555344000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5b98244414900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5083445544800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a46ea7548474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004996420474d58000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b50a66884a4f4500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a567d04c494e4b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c95ef624d4f4f5f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000004bc903fcc88e0504e4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000465000504e475f415641585f4554485f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009e04fb368504e475f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000003932b2e0c9c4e504e475f415641585f555344545f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000003962056d8f90a505450000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005896e8514900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de3bf544a5f415641585f4254435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000034028e4036e4c5544a5f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009f9fa17d0544a5f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000429739dba837f544a5f415641585f555344545f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000003d344478fb070544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000da1f286055534443000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f4f4db55534454000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f767a058415641000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002924fbc59414b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a27971e4e5959415633534131000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c0c6e7759595f414156455f415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c0c6e7759595f474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005d17ea659595f504e475f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000b05c34b9e59595f504e475f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000040a2a28137d9559595f5054505f7341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000756a688b59595f544a5f415641585f4554485f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa59cb72159595f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000048be9eb8821c859595f544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000ea816a857341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f8153de01871295a6100000002000002308463c6e07bdef53fbcb604c3565ec5ceed84e89259a1451f8d5888a7d4a99317c43bbcf57009693a358a2f41b0692145542bf18d36b180d7e3956b2ce3aaddc1c41564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000067ec5628425443000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000293428d3b0042555344000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5926844414900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5083445544800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a464a14a4474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004996420474d58000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b512099c4a4f4500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a62af04c494e4b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c964bc44d4f4f5f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000004bd1994d78353504e4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000465000504e475f415641585f4554485f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009e1a68ec3504e475f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000039392d46ec986504e475f415641585f555344545f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000003968852fc36d2505450000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005896e8514900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de63e544a5f415641585f4254435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000034087aaad9cad6544a5f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009fb546e29544a5f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000429ec4969cb45544a5f415641585f555344545f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000003d3b33066ff4b544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000da37dc9755534443000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f4f4db55534454000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f767a058415641000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002924fbc59414b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a27971e4e5959415633534131000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c24e97059595f414156455f415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c24e97059595f474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005d17ea659595f504e475f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000b0745edb159595f504e475f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000040a9f48a0a5f459595f5054505f7341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000756a688b59595f544a5f415641585f4554485f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa7122c8e59595f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000048c6dbe06856859595f544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000ea9bf9c57341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f8153de01871295a610000000200000239d0df82b13047cc49b085c6fe1310b4abcc3d2d4f3886b5ff2425221a5ed6cd37788e8353faa0d57b06033c100c8cd329d379dd0d7a6f7dfc9ed51a551962c7b1b41564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000067c89a804254430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002933d6e477c42555344000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5b98244414900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5083445544800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a4651b7b8474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004996420474d58000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b50a66884a4f4500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a5b6044c494e4b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c949b334d4f4f5f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000004bc903fcc88e0504e4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000465000504e475f415641585f4554485f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009e04fb368504e475f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000003932b2e0c9c4e504e475f415641585f555344545f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000003962056d8f90a505450000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005896e8514900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de3bf544a5f415641585f4254435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000034028e4036e4c5544a5f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009f9fa17d0544a5f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000429739dba837f544a5f415641585f555344545f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000003d344478fb070544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000da1f286055534443000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f4f4db55534454000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f767a058415641000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002924fbc59414b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a27971e4e5959415633534131000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c0c6e8959595f414156455f415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c0c6e8959595f474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005d17ea659595f504e475f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000b05c34b9e59595f504e475f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000040a2a28137d9559595f5054505f7341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000756a688b59595f544a5f415641585f4554485f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa59cb72159595f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000048be9eb8821c859595f544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000ea816a857341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f8153de01871295a61000000020000023ecfc1b788869cf6f5f90df0fb7192108483199195f72ba8b835bebcc6d379a372fa98dc56e74456e5c6dcbc4ca567e7a09b4252081e06f59325af337ad6a9e901c0003302e302e31392372656473746f6e652d6176616c616e6368652d70726f6400001e000002ed57011e0000".as_bytes().to_vec();
        let mut mi = "0xcaa648b441564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000067c89a804254430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002933d6e477c42555344000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5b98344414900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5083445544800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a46ea7548474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004996420474d58000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b50a66884a4f4500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a567d04c494e4b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c95ef634d4f4f5f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000004bc903fcc88e0504e4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000465000504e475f415641585f4554485f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009e04fb368504e475f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000003932b2e0c9c4e504e475f415641585f555344545f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000003962056d8f90a505450000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005896e8514900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de3bf544a5f415641585f4254435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000034028e4036e4c6544a5f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009f9fa17d0544a5f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000429739dba837f544a5f415641585f555344545f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000003d344478fb070544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000da1f286055534443000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f4f4db55534454000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f767a058415641000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002924fbc59414b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a27971e4e5959415633534131000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c0c6e7759595f414156455f415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c0c6e7759595f474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005d17ea659595f504e475f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000b05c34b9e59595f504e475f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000040a2a28137d9559595f5054505f7341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000756a688b59595f544a5f415641585f4554485f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa59cb72159595f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000048be9eb8821c959595f544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000ea816a857341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f8153df01871295a6100000002000002208463c6e07bdef53fbcb604c3565ec5ceed84e89259a1451f8d5888a7d4a99317c43bbcf57009693a358a2f41b0692145542bf18d36b180d7e3956b2ce3aaddc1c41564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000067ec5628425443000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000293428d3b0042555344000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5926944414900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5083445544800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a464a14a4474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004996420474d58000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b512099c4a4f4500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a62af04c494e4b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c964bc44d4f4f5f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000004bd1994d78353504e4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000465000504e475f415641585f4554485f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009e1a68ec3504e475f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000039392d46ec986504e475f415641585f555344545f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000003968852fc36d2505450000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005896e8514900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de63e544a5f415641585f4254435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000034087aaad9cad7544a5f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009fb546e29544a5f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000429ec4969cb45544a5f415641585f555344545f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000003d3b33066ff4b544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000da37dc9755534443000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f4f4db55534454000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f767a058415641000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002924fbc59414b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a27971e4e5959415633534131000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c24e97059595f414156455f415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c24e97059595f474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005d17ea659595f504e475f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000b0745edb159595f504e475f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000040a9f48a0a5f459595f5054505f7341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000756a688b59595f544a5f415641585f4554485f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa7122c8e59595f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000048c6dbe06856859595f544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000ea9bf9c57341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f8153df01871295a610000000200000229d0df82b13047cc49b085c6fe1310b4abcc3d2d4f3886b5ff2425221a5ed6cd37788e8353faa0d57b06033c100c8cd329d379dd0d7a6f7dfc9ed51a551962c7b1b41564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000067c89a804254430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002933d6e477c42555344000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5b98344414900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f5083445544800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a4651b7b8474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004996420474d58000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b50a66884a4f4500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a5b6044c494e4b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c949b334d4f4f5f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000004bc903fcc88e0504e4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000465000504e475f415641585f4554485f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009e04fb368504e475f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000003932b2e0c9c4e504e475f415641585f555344545f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000000003962056d8f90a505450000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005896e8514900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de3bf544a5f415641585f4254435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000034028e4036e4c6544a5f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009f9fa17d0544a5f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000429739dba837f544a5f415641585f555344545f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000003d344478fb070544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000da1f286055534443000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f4f4db55534454000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f767a058415641000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002924fbc59414b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a27971e4e5959415633534131000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c0c6e8959595f414156455f415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006c0c6e8959595f474c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005d17ea659595f504e475f415641585f4554485f4c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000b05c34b9e59595f504e475f415641585f555344435f4c500000000000000000000000000000000000000000000000000000000000000000000000000000040a2a28137d9559595f5054505f7341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000756a688b59595f544a5f415641585f4554485f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa59cb72159595f544a5f415641585f555344435f4c50000000000000000000000000000000000000000000000000000000000000000000000000000000048be9eb8821c959595f544a5f415641585f73415641585f4c500000000000000000000000000000000000000000000000000000000000000000000000000000000000ea816a857341564158000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f8153df01871295a61000000020000022ecfc1b788869cf6f5f90df0fb7192108483199195f72ba8b835bebcc6d379a372fa98dc56e74456e5c6dcbc4ca567e7a09b4252081e06f59325af337ad6a9e901c0003302e302e31392372656473746f6e652d6176616c616e6368652d70726f6400001e000002ed57011e0000".as_bytes().to_vec();
        let res_dp = compare_delta_prime_call_data_to_mine(
            &mut dp
        );
        let res_mine = compare_delta_prime_call_data_to_mine(
            &mut mi
        );


        let res = compare_data_extraction_results(&res_dp, &res_mine);

        println!("{}", res);
    }
}
