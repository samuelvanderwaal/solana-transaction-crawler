use rayon::prelude::*;
// use serde::Serialize;
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{
    EncodedConfirmedTransaction, EncodedTransaction, UiCompiledInstruction, UiMessage,
    UiTransactionEncoding,
};
use std::{
    collections::{HashMap, HashSet},
    // collections::HashSet,
    str::FromStr,
    sync::{Arc, Mutex},
};
use tokio::sync::Semaphore;
// use tokio::sync::Semaphore;

use crate::{errors::CrawlError, filters::*};

// Public API

pub struct IxAccount {
    name: String,
    index: usize,
}

impl IxAccount {
    pub fn new(name: &str, index: usize) -> Self {
        IxAccount {
            name: name.to_string(),
            index,
        }
    }
}

pub struct Crawler {
    client: Arc<RpcClient>,
    address: Pubkey,
    tx_filters: Vec<Box<dyn TxFilter + Send + Sync>>,
    ix_filters: Vec<Box<dyn IxFilter + Send + Sync>>,
    account_indices: Vec<IxAccount>,
}

impl Crawler {
    pub fn new(client: RpcClient, address: Pubkey) -> Self {
        Crawler {
            client: Arc::new(client),
            address,
            tx_filters: Vec::new(),
            ix_filters: Vec::new(),
            account_indices: Vec::new(),
        }
    }

    pub fn add_tx_filter<F: TxFilter + 'static + Send + Sync>(&mut self, filter: F) -> &mut Self {
        self.tx_filters.push(Box::new(filter));
        self
    }

    pub fn add_ix_filter<F: IxFilter + 'static + Send + Sync>(&mut self, filter: F) -> &mut Self {
        self.ix_filters.push(Box::new(filter));
        self
    }

    pub fn add_account_index(&mut self, index: IxAccount) -> &mut Self {
        self.account_indices.push(index);
        self
    }

    pub fn account_indices(&mut self, indices: Vec<IxAccount>) -> &mut Self {
        self.account_indices = indices;
        self
    }

    pub async fn run(self) -> Result<CrawledAccounts, CrawlError> {
        let signatures = self.get_all_signatures_for_id().await?;
        let transactions = self.get_transactions_from_signatures(signatures).await?;

        println!("transactions: {:?}", transactions.len());

        let filtered_transactions: Vec<&EncodedConfirmedTransaction> = transactions
            .iter()
            .filter(|tx| self.tx_filters.iter().all(|filter| filter.filter(tx)))
            .collect();

        println!("filtered tranasctions: {:?}", filtered_transactions.len());

        let ix_accounts = Arc::new(Mutex::new(HashMap::new()));

        filtered_transactions.par_iter().for_each(|tx| {
            let (account_keys, instructions) = match tx.transaction.transaction {
                EncodedTransaction::Json(ref ui_tx) => match &ui_tx.message {
                    UiMessage::Raw(raw_message) => {
                        (&raw_message.account_keys, &raw_message.instructions)
                    }
                    _ => panic!("not a raw message"),
                },
                _ => panic!("Not JSON encoded transaction"),
            };

            let filtered_instructions: Vec<&UiCompiledInstruction> = instructions
                .iter()
                .filter(|ix| {
                    self.ix_filters
                        .iter()
                        .all(|filter| filter.filter(ix, account_keys.clone()))
                })
                .collect();

            for ix in filtered_instructions {
                for a in self.account_indices.iter() {
                    let address = &account_keys[ix.accounts[a.index] as usize];
                    let mut ix_accounts = ix_accounts.lock().unwrap();
                    let ix_account = ix_accounts
                        .entry(a.name.to_string())
                        .or_insert(HashSet::new());
                    ix_account.insert(address.to_string());
                }
            }
        });

        let crawled_accounts = Arc::try_unwrap(ix_accounts).unwrap().into_inner().unwrap();

        Ok(crawled_accounts)
    }
}

pub type CrawledAccounts = HashMap<String, HashSet<String>>;

// Private methods
impl Crawler {
    async fn get_all_signatures_for_id(&self) -> Result<Vec<Signature>, CrawlError> {
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
            let sigs = self
                .client
                .get_signatures_for_address_with_config(&self.address, config)
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

    async fn get_transactions_from_signatures(
        &self,
        signatures: Vec<Signature>,
    ) -> Result<Vec<EncodedConfirmedTransaction>, CrawlError> {
        let mut transactions = Vec::new();
        // let errors = Vec::new();

        let mut tx_tasks = Vec::new();

        // Create a Semaphore to limit the number of concurrent requests.
        let sem = Arc::new(Semaphore::new(1000));

        for signature in signatures {
            let permit = Arc::clone(&sem).acquire_owned().await.unwrap();
            let client = self.client.clone();
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
}

async fn get_transaction(
    client: Arc<RpcClient>,
    signature: Signature,
) -> Result<EncodedConfirmedTransaction, CrawlError> {
    let transaction = client
        .get_transaction(&signature, UiTransactionEncoding::Json)
        .map_err(|err| CrawlError::ClientError(err.kind))?;

    Ok(transaction)
}
