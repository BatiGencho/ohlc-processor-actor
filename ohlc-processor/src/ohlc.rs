use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

/// State class for dynamic ohlc prices and timestamps
#[derive(Clone, Default, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OHLCState {
    pub prices: Vec<f64>,
    pub timestamps: Vec<u64>,
    pub time_frame: u64,
    pub current_rolling_ohlc: RollingOHLC,
}

impl OHLCState {
    /// Constructs a new OHLCState with a given timeframe
    pub fn new(time_frame: u64) -> Self {
        Self {
            prices: vec![],
            timestamps: vec![],
            time_frame,
            current_rolling_ohlc: RollingOHLC::default(),
        }
    }

    /// Dynamically adds a price and a timestamp to the OHLC State
    pub fn add_price_timestamp(&mut self, ticker: &str, price: f64, timestamp: u64) {
        self.prices.push(price);
        self.timestamps.push(timestamp);

        let start_index = self
            .timestamps
            .iter()
            .position(|&x| x > timestamp - self.time_frame)
            .unwrap_or(0);
        let end_index = self.timestamps.len() - 1;

        let prices_window = &self.prices[start_index..=end_index];
        let timestamps_window = &self.timestamps[start_index..=end_index];

        self.current_rolling_ohlc.ticker = Some(ticker.to_string());
        self.current_rolling_ohlc.open = prices_window[0];
        self.current_rolling_ohlc.high = prices_window
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        self.current_rolling_ohlc.low = prices_window.iter().cloned().fold(f64::INFINITY, f64::min);
        self.current_rolling_ohlc.close = prices_window[prices_window.len() - 1];
        self.current_rolling_ohlc.timestamp_start = timestamps_window[0];
        self.current_rolling_ohlc.timestamp_end = timestamps_window[timestamps_window.len() - 1];
    }

    /// Returns the current ohlc
    pub fn get_current_ohlcv(&self) -> RollingOHLC {
        self.current_rolling_ohlc.clone()
    }
}

/// This struct is primarily used for serializing the text output
/// The serialized output should look like this:
/// ```json
/// {"e":"bookTicker","u":1875301568520,"s":"TURBOUSDT","b":"0.3261","B":"226654.3","a":"0.3262","A":"75762.5","T":1662022800005,"E":1662022800010}
///```
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RollingOHLC {
    #[serde(rename(serialize = "symbol"))]
    pub ticker: Option<String>,
    #[serde(serialize_with = "float_as_string")]
    pub open: f64,
    #[serde(serialize_with = "float_as_string")]
    pub high: f64,
    #[serde(serialize_with = "float_as_string")]
    pub low: f64,
    #[serde(serialize_with = "float_as_string")]
    pub close: f64,
    #[serde(skip_serializing)]
    pub timestamp_start: u64,
    #[serde(rename(serialize = "timestamp"))]
    pub timestamp_end: u64,
}

/// A custom serializer formatting float as strings with a given precision
fn float_as_string<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{:.6}", value);
    serializer.serialize_str(&s)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_ohlc_state() {
        let ticker = "BTCUSD".to_string();
        let time_frame = 1 * 3600 * 1000; //1 hour in millis
        let mut state = OHLCState::new(time_frame);
        state.add_price_timestamp(&ticker, 10.0, 1604263400000);
        state.add_price_timestamp(&ticker, 12.5, 1604263460000);
        state.add_price_timestamp(&ticker, 11.0, 1604263520000);
        let ohlcv = state.get_current_ohlcv();
        assert_eq!(ohlcv.open, 10.0);
        assert_eq!(ohlcv.high, 12.5);
        assert_eq!(ohlcv.low, 10.0);
        assert_eq!(ohlcv.close, 11.0);
        assert_eq!(ohlcv.timestamp_start, 1604263400000);
        assert_eq!(ohlcv.timestamp_end, 1604263520000);

        state.add_price_timestamp(&ticker, 15.0, 1604263580000);
        let ohlcv = state.get_current_ohlcv();
        assert_eq!(ohlcv.open, 10.0);
        assert_eq!(ohlcv.high, 15.0);
        assert_eq!(ohlcv.low, 10.0);
        assert_eq!(ohlcv.close, 15.0);
        assert_eq!(ohlcv.timestamp_start, 1604263400000);
        assert_eq!(ohlcv.timestamp_end, 1604263580000);
    }

    #[test]
    fn test_rolling_ohlc_serialization() {
        let assert_serialized = r#"{"symbol":"TURBOUSDT","open":"101.300000","high":"208.900000","low":77.700000","close":"89.700000","timestamp":1662022800010}"#;
        let rolling_ohlc = RollingOHLC {
            ticker: Some("TURBOUSDT".to_string()),
            open: 101.3,
            high: 208.9,
            low: 77.7,
            close: 89.7,
            timestamp_end: 1662022800010,
            timestamp_start: 1662022800005,
        };
        let serialized_ohlc = serde_json::to_string(&rolling_ohlc).unwrap();
        assert_eq!(assert_serialized, serialized_ohlc);
    }
}
