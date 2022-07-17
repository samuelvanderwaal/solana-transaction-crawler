use solana_transaction_status::{
    EncodedConfirmedTransaction, EncodedTransaction, UiCompiledInstruction, UiMessage,
};

const CM_PROGRAM_ID: &str = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ";
const CM1_PROGRAM_ID: &str = "cndyAnrLdpjq1Ssp1z8xxDsB8dxe7u4HL5Nxi2K5WXZ";

pub fn cmv1_tx_filter(tx: &EncodedConfirmedTransaction) -> bool {
    match &tx.transaction.transaction {
        EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
            UiMessage::Raw(msg) => msg.account_keys.contains(&CM1_PROGRAM_ID.to_string()),
            _ => panic!("not a raw message"),
        },
        _ => panic!(""),
    }
}

pub fn cmv2_tx_filter(tx: &EncodedConfirmedTransaction) -> bool {
    match &tx.transaction.transaction {
        EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
            UiMessage::Raw(msg) => msg.account_keys.contains(&CM_PROGRAM_ID.to_string()),
            _ => panic!("not a raw message"),
        },
        _ => panic!(""),
    }
}

pub fn successful_txs_filter(tx: &EncodedConfirmedTransaction) -> bool {
    match &tx.transaction.meta {
        Some(meta) => meta.err.is_none(),
        None => false,
    }
}

// Candy Machine V1 always has 14 accounts:
// https://github.com/metaplex-foundation/the-graveyard/blob/5a523a8160ee413c54cb076a573c3d740b42ba84/nft-candy-machine/src/lib.rs#L407
pub fn cmv1_ix_len(ix: &&UiCompiledInstruction) -> bool {
    ix.accounts.len() == 14
}

// Candy Machine V2 always has at least 16 accounts, but can have more due to extra settings.
pub fn cmv2_ix_len(ix: &&UiCompiledInstruction) -> bool {
    ix.accounts.len() >= 16
}

pub fn ix_contains_data(ix: &&UiCompiledInstruction) -> bool {
    !ix.data.is_empty()
}
