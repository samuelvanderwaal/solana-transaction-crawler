use rayon::prelude::*;
use serde::Serialize;
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{
    EncodedConfirmedTransaction, EncodedTransaction, UiCompiledInstruction, UiMessage,
    UiTransactionEncoding,
};
use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, Mutex},
};
use tokio::sync::Semaphore;

use super::filters::*;
use crate::errors::CrawlError;

const CMV1_PROGRAM_ID: &str = "cndyAnrLdpjq1Ssp1z8xxDsB8dxe7u4HL5Nxi2K5WXZ";
const CMV2_PROGRAM_ID: &str = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CrawledAccounts {
    pub mint_addresses: HashSet<String>,
    pub metadata_addresses: HashSet<String>,
}

pub async fn get_cmv1_mints(
    client: RpcClient,
    candy_machine_id: &Pubkey,
) -> Result<CrawledAccounts, CrawlError> {
    let client = Arc::new(client);

    println!("Getting all signatures for {candy_machine_id}");
    let signatures = get_all_signatures_for_id(client.clone(), candy_machine_id).await?;

    println!("Getting transactions from signatures");
    let transactions = get_transactions_from_signatures(client, signatures).await?;

    println!("Filtering transactions for candy machine v1 instructions");
    let filtered_txs = transactions
        .into_par_iter()
        .filter(cmv1_tx_filter)
        .filter(successful_txs_filter)
        .collect::<Vec<_>>();

    let mint_addresses = Arc::new(Mutex::new(HashSet::new()));
    let metadata_addresses = Arc::new(Mutex::new(HashSet::new()));

    println!("Finding all 'mintNft' instructions");
    filtered_txs.par_iter().for_each(|tx| {
        let (account_keys, instructions) = match tx.transaction.transaction {
            EncodedTransaction::Json(ref ui_tx) => match &ui_tx.message {
                UiMessage::Raw(raw_message) => {
                    (&raw_message.account_keys, &raw_message.instructions)
                }
                _ => panic!("not a raw message"),
            },
            _ => panic!("Not JSON encoded transaction"),
        };

        // Candy machine v1 has at least 13 accounts, v2 has at least 16.

        let filtered_instructions: Vec<&UiCompiledInstruction> = instructions
            .iter()
            .filter(|ix| account_keys[ix.program_id_index as usize] == *CMV1_PROGRAM_ID)
            .filter(cmv1_ix_len)
            .filter(|ix| account_keys[ix.accounts[1] as usize] == candy_machine_id.to_string())
            .collect();

        // Should only be one or zero ix that meets these requirements.
        match filtered_instructions.len() {
            0 => (),
            1 => {
                let ix = filtered_instructions[0];
                metadata_addresses
                    .lock()
                    .unwrap()
                    .insert(account_keys[ix.accounts[4] as usize].clone());
                mint_addresses
                    .lock()
                    .unwrap()
                    .insert(account_keys[ix.accounts[5] as usize].clone());
            }
            _ => panic!(
                "Expected zero or one instruction, got {} on tx: {:?}",
                filtered_instructions.len(),
                tx
            ),
        }
    });

    println!("Collating addresses");
    let mint_addresses = Arc::try_unwrap(mint_addresses)
        .unwrap()
        .into_inner()
        .unwrap();
    let metadata_addresses = Arc::try_unwrap(metadata_addresses)
        .unwrap()
        .into_inner()
        .unwrap();

    Ok(CrawledAccounts {
        mint_addresses,
        metadata_addresses,
    })
}

pub async fn get_cmv2_mints(
    client: RpcClient,
    candy_machine_id: &Pubkey,
) -> Result<CrawledAccounts, CrawlError> {
    let client = Arc::new(client);

    println!("Getting all signatures for {candy_machine_id}");
    let signatures = get_all_signatures_for_id(client.clone(), candy_machine_id).await?;

    println!("Getting transactions from signatures");
    let transactions = get_transactions_from_signatures(client, signatures).await?;

    println!("Filtering transactions for candy machine v2 instructions");
    let filtered_txs = transactions
        .into_iter()
        .filter(cmv2_tx_filter)
        .filter(successful_txs_filter)
        .collect::<Vec<_>>();

    let mint_addresses = Arc::new(Mutex::new(HashSet::new()));
    let metadata_addresses = Arc::new(Mutex::new(HashSet::new()));

    println!("Finding all 'mintNft' instructions");
    filtered_txs.par_iter().for_each(|tx| {
        let (account_keys, instructions) = match tx.transaction.transaction {
            EncodedTransaction::Json(ref ui_tx) => match &ui_tx.message {
                UiMessage::Raw(raw_message) => {
                    (&raw_message.account_keys, &raw_message.instructions)
                }
                _ => panic!("not a raw message"),
            },
            _ => panic!("Not JSON encoded transaction"),
        };

        let filtered_instructions: Vec<_> = instructions
            .iter()
            .filter(|ix| account_keys[ix.program_id_index as usize] == *CMV2_PROGRAM_ID)
            .filter(cmv2_ix_len)
            // .filter(|ix| account_keys[ix.accounts[0] as usize] == candy_machine_id.to_string())
            .collect();

        // Should only be one or zero ix that meets these requirements.
        match filtered_instructions.len() {
            0 => (),
            1 => {
                let ix = filtered_instructions[0];
                metadata_addresses
                    .lock()
                    .unwrap()
                    .insert(account_keys[ix.accounts[4] as usize].clone());
                mint_addresses
                    .lock()
                    .unwrap()
                    .insert(account_keys[ix.accounts[5] as usize].clone());
            }
            _ => panic!(
                "Expected zero or one instruction, got {} on tx: {:?}",
                filtered_instructions.len(),
                tx
            ),
        }
    });

    println!("Collating addresses");
    let mint_addresses = Arc::try_unwrap(mint_addresses)
        .unwrap()
        .into_inner()
        .unwrap();
    let metadata_addresses = Arc::try_unwrap(metadata_addresses)
        .unwrap()
        .into_inner()
        .unwrap();

    Ok(CrawledAccounts {
        mint_addresses,
        metadata_addresses,
    })
}

async fn get_transactions_from_signatures(
    client: Arc<RpcClient>,
    signatures: Vec<Signature>,
) -> Result<Vec<EncodedConfirmedTransaction>, CrawlError> {
    let mut transactions = Vec::new();
    // let errors = Vec::new();

    let mut tx_tasks = Vec::new();

    // Create a Semaphore to limit the number of concurrent requests.
    let sem = Arc::new(Semaphore::new(1000));

    for signature in signatures {
        let permit = Arc::clone(&sem).acquire_owned().await.unwrap();
        let client = client.clone();
        tx_tasks.push(tokio::spawn(async move {
            // Move permit into the closure so it is dropped when the task is dropped.
            let _permit = permit;
            get_transaction(client, signature).await
        }));
    }

    for task in tx_tasks {
        let res = task.await.unwrap();
        if let Ok(tx) = res {
            transactions.push(tx);
        }
    }

    Ok(transactions)
}

async fn get_transaction(
    client: Arc<RpcClient>,
    signature: Signature,
) -> Result<EncodedConfirmedTransaction, CrawlError> {
    let client = client;
    let transaction = client
        .get_transaction(&signature, UiTransactionEncoding::Json)
        .map_err(|err| CrawlError::ClientError(err.kind))?;

    Ok(transaction)
}

async fn get_all_signatures_for_id(
    client: Arc<RpcClient>,
    candy_machine_id: &Pubkey,
) -> Result<Vec<Signature>, CrawlError> {
    let mut signatures = Vec::new();

    // Initial config
    let mut before = None;
    let until = None;
    let limit = Some(1000);
    let commitment = Some(CommitmentConfig::finalized());
    let mut retries = 0u8;

    loop {
        let config = GetConfirmedSignaturesForAddress2Config {
            before,
            until,
            limit,
            commitment,
        };
        let sigs = client
            .get_signatures_for_address_with_config(candy_machine_id, config)
            .map_err(|err| CrawlError::ClientError(err.kind))?;

        let last_sig = match sigs.last() {
            Some(sig) => sig,
            None => break,
        };

        let last_sig = Signature::from_str(&last_sig.signature)
            .map_err(|err| CrawlError::SignatureParseFailed(err.to_string()))?;

        // Loop until we reach the last batch of signatures.
        match sigs.len() {
            1000 => {
                before = Some(last_sig);
                signatures.extend(sigs);
                retries = 0;
            }
            0 => {
                if retries < 10 {
                    retries += 1;
                    continue;
                } else {
                    break;
                }
            }
            _ => {
                signatures.extend(sigs);
                break;
            }
        }
    }

    let signatures = signatures
        .into_iter()
        .map(|sig| sig.signature)
        .map(|s| Signature::from_str(&s).unwrap())
        .collect();

    Ok(signatures)
}
