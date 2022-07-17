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

pub struct Cmv2TxFilter;

impl TxFilter for Cmv2TxFilter {
    fn filter(&self, tx: &EncodedConfirmedTransaction) -> bool {
        match &tx.transaction.transaction {
            EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
                UiMessage::Raw(msg) => msg.account_keys.contains(&CMV2_PROGRAM_ID.to_string()),
                _ => panic!("not a raw message"),
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
                _ => panic!("not a raw message"),
            },
            _ => panic!("not Json encoded"),
        }
    }
}
