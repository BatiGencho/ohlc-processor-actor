use crate::error::Error;
use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

/// Rust structure used to deserialize the text file input
/// # Example format of the text file
/// ```json
/// {"e":"bookTicker","u":1875301568520,"s":"TURBOUSDT","b":"0.3261","B":"226654.3","a":"0.3262","A":"75762.5","T":1662022800005,"E":1662022800010}
///```
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PriceDataEntity {
    /// The event name
    #[serde(rename(deserialize = "e"))]
    pub event_name: String,
    /// The event category
    #[serde(rename(deserialize = "u"))]
    pub cat: u64,
    /// The order ticker
    #[serde(rename(deserialize = "s"))]
    pub ticker: String,
    #[serde(
        deserialize_with = "deserialize_number_from_string",
        rename(deserialize = "b")
    )]
    /// The bid price
    pub bid_price: f64,
    #[serde(
        deserialize_with = "deserialize_number_from_string",
        rename(deserialize = "B")
    )]
    // The bid quantity
    pub bid_quantity: f64,
    #[serde(
        deserialize_with = "deserialize_number_from_string",
        rename(deserialize = "a")
    )]
    /// The ask price
    pub ask_price: f64,
    #[serde(
        deserialize_with = "deserialize_number_from_string",
        rename(deserialize = "A")
    )]
    /// The ask quantity
    pub ask_quantity: f64,
    #[serde(rename(deserialize = "T"))]
    /// The start timestamp of the interval
    pub timestamp_start: u64,
    /// The end timestamp of the interval
    #[serde(rename(deserialize = "E"))]
    pub timestamp_end: u64,
}

/// Reads a text file line by line collecting the lines in a vector
pub async fn read_lines<'a, P>(path: P) -> Result<Vec<String>, Error>
where
    P: AsRef<Path>,
{
    let file = File::open(path).await.map_err(Error::OpenDataFile)?;
    let _ = file.metadata().await.map_err(Error::ReadMeta)?;
    let mut reader = BufReader::new(file);
    let mut vec_t = vec![];
    loop {
        let mut line = String::new();
        let bytes_read = reader
            .read_line(&mut line)
            .await
            .map_err(Error::ReadDataFile)?;
        if bytes_read == 0 {
            break;
        }
        vec_t.push(line);
    }
    Ok(vec_t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialization() {
        let serialized = r#"{"e":"bookTicker","u":1875301568520,"s":"TURBOUSDT","b":"0.3261","B":"226654.3","a":"0.3262","A":"75762.5","T":1662022800005,"E":1662022800010}"#;
        let deserialized: PriceDataEntity = serde_json::from_str(&serialized).unwrap();
        let assert_instance = PriceDataEntity {
            event_name: "bookTicker".to_string(),
            cat: 1875301568520,
            ticker: "TURBOUSDT".to_string(),
            bid_price: 0.3261,
            bid_quantity: 226654.3,
            ask_price: 0.3262,
            ask_quantity: 75762.5,
            timestamp_end: 1662022800010,
            timestamp_start: 1662022800005,
        };
        assert_eq!(deserialized, assert_instance);
    }
}
