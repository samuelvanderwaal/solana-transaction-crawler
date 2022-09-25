use super::*;

/// This filter passes through all successful transactions, rejecting any with errors.
pub struct SuccessfulTxFilter;

impl TxFilter for SuccessfulTxFilter {
    fn filter(&self, tx: &EncodedConfirmedTransactionWithStatusMeta) -> bool {
        match &tx.transaction.meta {
            Some(meta) => meta.err.is_none(),
            None => false,
        }
    }
}

/// This filter passes through all transactions that do not have the CMV2 bot tax message in the logs.
/// It is used to filter out transactions that succeeded but were bot taxed, so did not produce any NFT accounts.
pub struct CmV2BotTaxTxFilter;

impl TxFilter for CmV2BotTaxTxFilter {
    fn filter(&self, tx: &EncodedConfirmedTransactionWithStatusMeta) -> bool {
        // Filter out bot tax transactions, pass through everything else.
        match &tx.transaction.meta {
            Some(meta) => {
                if let Some(messages) = &meta.log_messages {
                    !messages
                        .iter()
                        .any(|m| m.contains(&CMV2_BOT_TAX_MSG.to_string()))
                } else {
                    true
                }
            }
            None => true,
        }
    }
}

/// This filter passes through all transactions that have the Candy Machine V2 progarm id.
pub struct Cmv2TxFilter;

impl TxFilter for Cmv2TxFilter {
    fn filter(&self, tx: &EncodedConfirmedTransactionWithStatusMeta) -> bool {
        match &tx.transaction.transaction {
            EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
                UiMessage::Raw(msg) => msg.account_keys.contains(&CMV2_PROGRAM_ID.to_string()),
                UiMessage::Parsed(msg) => msg
                    .account_keys
                    .iter()
                    .map(|a| a.pubkey.clone())
                    .any(|a| a == *CMV2_PROGRAM_ID),
            },
            _ => panic!("not Json encoded"),
        }
    }
}

/// This filter allows specifying a custom program id and passes through any transactions that have it.
pub struct TxHasProgramId {
    program_id: String,
}

impl TxHasProgramId {
    pub fn new(program_id: &str) -> Self {
        TxHasProgramId {
            program_id: program_id.to_string(),
        }
    }
}

impl TxFilter for TxHasProgramId {
    fn filter(&self, tx: &EncodedConfirmedTransactionWithStatusMeta) -> bool {
        match &tx.transaction.transaction {
            EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
                UiMessage::Raw(msg) => msg.account_keys.contains(&self.program_id),
                UiMessage::Parsed(msg) => msg
                    .account_keys
                    .iter()
                    .map(|a| a.pubkey.clone())
                    .any(|a| a == self.program_id),
            },
            _ => panic!("not Json encoded"),
        }
    }
}

/// This filter passes through all transactions where the provided address is a signer.
pub struct TxHasSigner {
    address: String,
}

impl TxHasSigner {
    pub fn new(address: &str) -> Self {
        TxHasSigner {
            address: address.to_string(),
        }
    }
}

impl TxFilter for TxHasSigner {
    fn filter(&self, tx: &EncodedConfirmedTransactionWithStatusMeta) -> bool {
        match &tx.transaction.transaction {
            EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
                UiMessage::Raw(_) => panic!("Not a parsed tx message"),
                UiMessage::Parsed(msg) => msg
                    .account_keys
                    .iter()
                    .any(|a| a.pubkey == self.address && a.signer),
            },
            _ => panic!("not Json encoded"),
        }
    }
}
