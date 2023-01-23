## Usage

The project is split into 2 crates:

- `ohlc-processor` which is the main library processor crate with its interface
- `ohlc-runner` which is the binary tester crate

The test files could be tested by using the `ohlc-runner` and calling inside the `ohlc-runner` folder:

`cargo run -- --in-file [path_to_test_file.txt] --out-file [some_file_name]`

The generated output will be under `some_file_name`. [path_to_test_file.txt] refers to the test files provided in the data folder in the project.

The `ohlc-processor` crate has a simple usage:

```rust

    // define timeframe
    let timeframe_millis = 5 * 60 * 1000; // 5 minutes in milliseconds
    // define file path
    let in_file_path = Path::new("some_input_file.txt");
    // instantiate the processor and load the prices file
    let ohlc_processor = OHLCProcessor::new(timeframe_millis)
        .load_prices(&in_file_path)
        .await?
    // create a channel to receive computed ohlcs
    let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel::<RollingOHLC>();

    // spawn a tokio thread to receive from the channel receiver
    tokio::spawn(async move {
        while let Some(r) = out_rx.recv().await {
            let data = serde_json::to_string(&r).unwrap();
            println!("Received OHLC {:?}", data);
        }
    });

    // run the processor in the background
    ohlc_processor
        .run(out_tx)
        .await?;
```