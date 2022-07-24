use super::*;

pub struct SuccessfulTxFilter;

impl TxFilter for SuccessfulTxFilter {
    fn filter(&self, tx: &EncodedConfirmedTransaction) -> bool {
        match &tx.transaction.meta {
            Some(meta) => meta.err.is_none(),
            None => false,
        }
    }
}

pub struct CmV2BotTaxTxFilter;

impl TxFilter for CmV2BotTaxTxFilter {
    fn filter(&self, tx: &EncodedConfirmedTransaction) -> bool {
        // Filter out bot tax transactions, pass through everything else.
        match &tx.transaction.meta {
            Some(meta) => {
                if let Some(messages) = &meta.log_messages {
                    !messages.contains(&CMV2_BOT_TAX_MSG.to_string())
                } else {
                    true
                }
            }
            None => true,
        }
    }
}

pub struct Cmv2TxFilter;

impl TxFilter for Cmv2TxFilter {
    fn filter(&self, tx: &EncodedConfirmedTransaction) -> bool {
        match &tx.transaction.transaction {
            EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
                UiMessage::Raw(msg) => msg.account_keys.contains(&CMV2_PROGRAM_ID.to_string()),
                UiMessage::Parsed(msg) => {
                    msg.account_keys
                        .iter()
                        .map(|a| a.pubkey.clone())
                        .any(|a| a == *CMV2_PROGRAM_ID)

                    // accounts.contains(&CMV2_PROGRAM_ID.to_string())
                }
            },
            _ => panic!("not Json encoded"),
        }
    }
}

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
    fn filter(&self, tx: &EncodedConfirmedTransaction) -> bool {
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
