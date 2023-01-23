use displaydoc::Display as DisplayDoc;
use thiserror::Error;

/// Error enum representing all errors returned by the library
#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// Open data file: {0}
    OpenDataFile(std::io::Error),
    /// Failed to read data file: {0}
    ReadDataFile(std::io::Error),
    /// Failed to read data file metadata: {0}
    ReadMeta(std::io::Error),
    /// No prices data loaded
    MissingPricesData,
    /// Tokio handle join Error
    TokioJoinError(tokio::task::JoinError),
}
