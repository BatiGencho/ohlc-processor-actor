use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;
use std::path::Path;
use ohlc_processor::ohlc::RollingOHLC;
use ohlc_processor::{
    processor::OHLCProcessor,
};

/// An Args parser
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct RunnerArgs {
    /// Path to the data file to read
    #[arg(short, long)]
    pub in_file: String,
    /// Name of the out file. File will be in .txt format in the root of the binary crate
    #[arg(short, long)]
    pub out_file: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = RunnerArgs::parse();
    println!("Data file path: {:?} ", args.in_file);

    let timeframe_millis = 5 * 60 * 1000; // 5 minutes in millisconds
    let in_file_path = Path::new(&args.in_file);
    let ohlc_processor = OHLCProcessor::new(timeframe_millis)
        .load_prices(&in_file_path)
        .await
        .context("Error loading prices data file content")?;
    let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel::<RollingOHLC>();

    tokio::spawn(async move {
        // create an out file
        let mut counter = 0;
        let mut file = std::fs::File::create(format!("{}.txt", args.out_file)).unwrap();

        while let Some(r) = out_rx.recv().await {
            counter += 1;
            let data = serde_json::to_string(&r).unwrap();
            println!("Price index = {:?}. OHLC = {:?}", counter, data);
            file.write_all(data.as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();
        }
    });

    ohlc_processor
        .run(out_tx)
        .await
        .context("Error in the OHLC processor")?;

    Ok(())
}
