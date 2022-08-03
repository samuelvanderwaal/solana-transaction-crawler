use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrawlError {
    #[error("no account data found")]
    MissingAccount(String),

    #[error("RPC call failed with error: {0} for value: {1}")]
    ClientError(String, String),

    #[error("failed to parse string into Pubkey")]
    PubkeyParseFailed(String),

    #[error("Failed to parse signature: {0}")]
    SignatureParseFailed(String),
}
