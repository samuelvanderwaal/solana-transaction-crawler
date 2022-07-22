use crate::constants::*;
use solana_transaction_status::{
    EncodedConfirmedTransaction, EncodedTransaction, UiMessage, UiParsedInstruction,
};

pub mod ix;
pub mod tx;

pub use ix::*;
pub use tx::*;

pub trait TxFilter {
    fn filter(&self, tx: &EncodedConfirmedTransaction) -> bool;
}

pub trait IxFilter {
    fn filter(&self, ix: &UiParsedInstruction) -> bool;
}
