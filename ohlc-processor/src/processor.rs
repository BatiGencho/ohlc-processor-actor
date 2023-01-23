use crate::{
    actor::{Message, OHLCActor, PriceActor},
    args::{read_lines, PriceDataEntity},
    error::Error,
    ohlc::RollingOHLC,
};
use std::path::Path;
use tokio::sync::mpsc::UnboundedSender;

/// A processor which streams computed OHLCs for a defined timeframe and given prices data file
pub struct OHLCProcessor {
    pub timeframe_millis: u64,
    pub input_data: Vec<PriceDataEntity>,
}

impl OHLCProcessor {
    /// Constructs a new instance of the `OHLCProcessor` and accepts a timeframe in milliseconds
    pub fn new(timeframe_millis: u64) -> Self {
        Self {
            timeframe_millis,
            input_data: vec![],
        }
    }

    /// Loads prices from a text file. Accepts a text file path
    pub async fn load_prices(mut self, prices_file_path: &Path) -> Result<Self, Error> {
        let prices_input = read_lines(prices_file_path)
            .await?
            .iter()
            .filter_map(|line| {
                let item = serde_json::from_str::<PriceDataEntity>(&line).ok();
                item
            })
            .collect::<Vec<_>>();
        println!("Loaded {:?} price items", prices_input.len());
        self.input_data = prices_input;
        Ok(self)
    }

    /// Runs the async processor. Returns a Receiver channel end into which the Rolling OHLCs shall be streamed
    pub async fn run(self, out_tx: UnboundedSender<RollingOHLC>) -> Result<(), Error> {
        if self.input_data.is_empty() {
            return Err(Error::MissingPricesData);
        }
        let (tx, rx) = tokio::sync::mpsc::channel::<Message>(self.input_data.len());
        //let (out_tx, out_rx) = tokio::sync::mpsc::unbounded_channel::<RollingOHLC>();

        let ohlc_actor = OHLCActor::new(rx, self.timeframe_millis);

        tokio::spawn(async move {
            for price_input in self.input_data.into_iter() {
                let out_tx = out_tx.clone();
                let price_actor = PriceActor::new(price_input, tx.clone(), out_tx);
                price_actor.send().await;
            }
        });

        tokio::spawn(async move {
            ohlc_actor.run().await;
        })
        .await
        .map_err(Error::TokioJoinError)?;

        Ok(())
    }
}
