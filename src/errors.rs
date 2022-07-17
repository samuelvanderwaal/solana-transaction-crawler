use solana_client::client_error::ClientErrorKind;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrawlError {
    #[error("no account data found")]
    MissingAccount(String),

    #[error("failed to get account data")]
    ClientError(ClientErrorKind),

    #[error("failed to parse string into Pubkey")]
    PubkeyParseFailed(String),

    #[error("Failed to parse signature: {0}")]
    SignatureParseFailed(String),
}
