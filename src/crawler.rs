use rayon::prelude::*;
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{
    EncodedConfirmedTransaction, EncodedTransaction, UiInstruction, UiMessage, UiParsedInstruction,
    UiTransactionEncoding,
};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::{Arc, Mutex},
};
use tokio::{sync::Semaphore, time::Instant};

use crate::{constants::*, errors::CrawlError, filters::*};

// Public API
pub type CrawledAccounts = HashMap<String, HashSet<String>>;

// Instruction Accounts represent the specific accounts users wish to retrieve from an instruction.
// For unparsed instructions the user must specify the account index and the name they wish to it be labeled.
// For parsed instructions the users must specify the actual name as it's represented in the instruction.
pub struct IxAccount {
    name: String,
    index: Option<usize>,
}

impl IxAccount {
    pub fn unparsed(name: &str, index: usize) -> Self {
        Self {
            name: name.to_string(),
            index: Some(index),
        }
    }
    pub fn parsed(name: &str) -> Self {
        Self {
            name: name.to_string(),
            index: None,
        }
    }
}

pub struct Crawler {
    client: Arc<RpcClient>,
    address: Pubkey,
    tx_filters: Vec<Box<dyn TxFilter + Send + Sync>>,
    ix_filters: Vec<Box<dyn IxFilter + Send + Sync>>,
    ix_or_filters: Vec<Box<dyn IxFilter + Send + Sync>>,
    account_indices: Vec<IxAccount>,
}

impl Crawler {
    pub fn new(client: RpcClient, address: Pubkey) -> Self {
        Crawler {
            client: Arc::new(client),
            address,
            tx_filters: Vec::new(),
            ix_filters: Vec::new(),
            ix_or_filters: Vec::new(),
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

    pub fn add_ix_or_filters<F: IxFilter + 'static + Send + Sync>(
        &mut self,
        filters: Vec<F>,
    ) -> &mut Self {
        filters
            .into_iter()
            .for_each(|filter| self.ix_or_filters.push(Box::new(filter)));
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
        let start = Instant::now();
        let signatures = self.get_all_signatures_for_id().await?;
        let sigs_time = Instant::now();
        println!(
            "Retrieved signatures {:?} in {:?}",
            signatures.len(),
            sigs_time - start
        );
        let transactions = self.get_transactions_from_signatures(signatures).await?;

        let tx_time = Instant::now();
        println!(
            "Retrieved transactions {:?} in {:?}, time since start: {:?}",
            transactions.len(),
            tx_time - sigs_time,
            tx_time - start
        );

        let filtered_transactions: Vec<&EncodedConfirmedTransaction> = transactions
            .iter()
            .filter(|tx| self.tx_filters.iter().all(|filter| filter.filter(tx)))
            .collect();

        println!("filtered tranasctions: {:?}", filtered_transactions.len());
        let filtered_tx_time = Instant::now();
        println!(
            "Filtered transactions in {:?}, time since start: {:?}",
            filtered_tx_time - tx_time,
            filtered_tx_time - start
        );

        let ix_accounts = Arc::new(Mutex::new(HashMap::new()));

        filtered_transactions.par_iter().for_each(|tx| {
            let mut instructions: Vec<&UiParsedInstruction> = match tx.transaction.transaction {
                EncodedTransaction::Json(ref ui_tx) => match &ui_tx.message {
                    UiMessage::Raw(_msg) => {
                        panic!("not a parsed message");
                    }
                    UiMessage::Parsed(msg) => msg
                        .instructions
                        .iter()
                        .map(|ix| match ix {
                            UiInstruction::Parsed(ix) => ix,
                            _ => panic!("not a parsed instruction"),
                        })
                        .collect::<Vec<&UiParsedInstruction>>(),
                },
                _ => panic!("Not JSON encoded transaction"),
            };

            // Get all inner instructions and add them to the instructions list.
            if let Some(meta) = &tx.transaction.meta {
                if let Some(inner_instructions) = &meta.inner_instructions {
                    let mut parsed_ixs = inner_instructions
                        .iter()
                        .map(|ix| &ix.instructions)
                        .flatten()
                        .map(|ix| match ix {
                            UiInstruction::Parsed(ix) => ix,
                            _ => panic!("not a parsed instruction"),
                        })
                        .collect::<Vec<&UiParsedInstruction>>();
                    instructions.append(&mut parsed_ixs);
                }
            }

            let filtered_instructions: Vec<&UiParsedInstruction> = instructions
                .into_iter()
                .filter(|ix| self.ix_filters.iter().all(|filter| filter.filter(ix)))
                .filter(|ix| self.ix_or_filters.iter().any(|filter| filter.filter(ix)))
                .collect();

            // Fetch accounts from instructions
            for ix in filtered_instructions {
                for a in self.account_indices.iter() {
                    match ix {
                        UiParsedInstruction::PartiallyDecoded(ix) => {
                            if let Some(index) = a.index {
                                let address = &ix.accounts[index];
                                let mut ix_accounts = ix_accounts.lock().unwrap();

                                let ix_account = ix_accounts
                                    .entry(a.name.to_string())
                                    .or_insert_with(HashSet::new);
                                ix_account.insert(address.to_string());
                            }
                        }
                        UiParsedInstruction::Parsed(ix) => {
                            if a.index.is_none() {
                                let pointer = format!("/info/{}", a.name);
                                let address_opt = ix.parsed.pointer(&pointer);
                                if let Some(address) = address_opt {
                                    let mut ix_accounts = ix_accounts.lock().unwrap();

                                    let address = address.as_str().unwrap().trim_matches('\\');

                                    let ix_account = ix_accounts
                                        .entry(a.name.to_string())
                                        .or_insert_with(HashSet::new);
                                    ix_account.insert(address.to_string());
                                }
                            }
                        }
                    }
                }
            }
        });
        let parse_ixs_time = Instant::now();
        println!(
            "Parsed instructions in {:?}, time since start: {:?}",
            parse_ixs_time - filtered_tx_time,
            parse_ixs_time - start
        );

        let crawled_accounts = Arc::try_unwrap(ix_accounts).unwrap().into_inner().unwrap();

        Ok(crawled_accounts)
    }
}

// Associated functions for common crawl patterns
impl Crawler {
    pub async fn get_cmv2_mints(
        client: RpcClient,
        candy_machine_pubkey: Pubkey,
    ) -> Result<CrawledAccounts, CrawlError> {
        let has_program_id = TxHasProgramId::new(CMV2_PROGRAM_ID);
        let ix_program_id = IxProgramIdFilter::new(CMV2_PROGRAM_ID);
        let ix_num_accounts = IxNumberAccounts::GreaterThanOrEqual(16);
        let metadata_account = IxAccount::unparsed("metadata_account", 4);
        let mint_account = IxAccount::unparsed("mint_account", 5);

        let mut crawler = Crawler::new(client, candy_machine_pubkey);
        crawler
            .add_tx_filter(has_program_id)
            .add_tx_filter(SuccessfulTxFilter)
            .add_tx_filter(CmV2BotTaxTxFilter)
            .add_ix_filter(ix_program_id)
            .add_ix_filter(ix_num_accounts)
            .add_account_index(metadata_account)
            .add_account_index(mint_account);

        crawler.run().await
    }

    pub async fn get_first_verified_creator_mints(
        client: RpcClient,
        creator: Pubkey,
    ) -> Result<CrawledAccounts, CrawlError> {
        // We're looking for all the create_master_edition and create_master_edition_v2 instructions and
        // getting the mint accounts from them.
        // Creating a master edition means it's a Metaplex NFT and not a SPL token or a Fungible Asset.
        // The CREATE_MASTER_EDITION_DATA constant has the base58 encoded data for a create_master_edition call
        // where the max_supply is set to be Some(0), so this also filters out master editions with prints.

        let has_program_id = TxHasProgramId::new(TOKEN_METADATA_PROGAM_ID);
        let ix_program_id = IxProgramIdFilter::new(TOKEN_METADATA_PROGAM_ID);
        let ix_data = IxDataFilter::new(CREATE_MASTER_EDITION_DATA);
        let ix_data2 = IxDataFilter::new(CREATE_MASTER_EDITION_V3_DATA);
        let mint = IxAccount::unparsed("mint", 1);

        let mut crawler = Crawler::new(client, creator);
        crawler
            .add_tx_filter(has_program_id)
            .add_tx_filter(SuccessfulTxFilter)
            .add_tx_filter(CmV2BotTaxTxFilter)
            .add_ix_filter(ix_program_id)
            .add_ix_or_filters(vec![ix_data, ix_data2])
            .add_account_index(mint);

        crawler.run().await
    }
}

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
        .get_transaction(&signature, UiTransactionEncoding::JsonParsed)
        .map_err(|err| CrawlError::ClientError(err.kind))?;

    Ok(transaction)
}
