use crate::args::PriceDataEntity;
use crate::ohlc::{OHLCState, RollingOHLC};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;

/// A transport message used by the `PriceActor` for communicating with the `OHLCActor`
#[derive(Debug)]
pub struct Message {
    pub data: PriceDataEntity,
    pub respond_to: oneshot::Sender<RollingOHLC>,
}

/// A `PriceActor` is an actor that communicates with the `OHLCActor` orchestrator by sending over price data
/// and replying over using a channel for acknowledgement
pub struct PriceActor {
    pub data: PriceDataEntity,
    pub sender: Sender<Message>,
    pub out_tx: UnboundedSender<RollingOHLC>,
}

impl PriceActor {
    /// Constructs a new `PriceActor`
    pub fn new(
        data: PriceDataEntity,
        sender: Sender<Message>,
        out_tx: UnboundedSender<RollingOHLC>,
    ) -> Self {
        Self {
            data,
            sender,
            out_tx,
        }
    }

    /// Sends the `Message` to the `PriceActor` and acknowledges
    pub async fn send(self) {
        let (send, recv) = oneshot::channel();
        let msg = Message {
            data: self.data,
            respond_to: send,
        };
        let _ = self.sender.send(msg).await;
        match recv.await {
            Err(e) => println!("Price actor receive error {}", e),
            Ok(rolling_ohlc) => {
                // TODO: print here the OHLC report!
                match self.out_tx.send(rolling_ohlc) {
                    Ok(_) => {
                        //println!("Ohlc processed");
                    }
                    Err(e) => {
                        eprintln!("Ohlc sending errored {:?}", e.to_string())
                    }
                }
            }
        }
    }
}

/// An actor structure reading messages from `PriceActor` and updating the current OHLC over a given time frame.
/// The OHLC state is sent back to the calling `PriceActor`.
#[derive(Debug)]
pub struct OHLCActor {
    pub receiver: Receiver<Message>,
    pub timeframe_millis: u64,
    pub price_data: HashMap<String, OHLCState>,
}

impl OHLCActor {
    /// Creates a new `OHLCActor`
    pub fn new(receiver: Receiver<Message>, timeframe_millis: u64) -> Self {
        Self {
            receiver,
            timeframe_millis,
            price_data: HashMap::new(),
        }
    }

    /// Processes an incoming message by updating and emitting the current ohlc
    pub fn handle_message(&mut self, message: Message) {
        let ticker = message.data.ticker.clone();
        let ticker_price_storage = self
            .price_data
            .entry(ticker.clone())
            .or_insert(OHLCState::new(self.timeframe_millis));

        ticker_price_storage.add_price_timestamp(
            &ticker,
            message.data.ask_price,
            message.data.timestamp_start,
        );
        let _ = message
            .respond_to
            .send(ticker_price_storage.get_current_ohlcv());
    }

    /// Runs the actor until completion
    pub async fn run(mut self) {
        println!("OHLC Actor is running");
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg);
        }
    }
}
