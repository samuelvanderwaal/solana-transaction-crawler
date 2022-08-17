use crate::constants::*;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiMessage, UiParsedInstruction,
};

pub mod ix;
pub mod tx;

pub use ix::*;
pub use tx::*;

/// This trait defines the interface for creating a filter that is applied to all transactions.
pub trait TxFilter {
    fn filter(&self, tx: &EncodedConfirmedTransactionWithStatusMeta) -> bool;
}

/// This trait defines the interface for creating a filter that is applied to all instructions.
pub trait IxFilter {
    fn filter(&self, ix: &UiParsedInstruction) -> bool;
}
