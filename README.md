## Solana Crawler

### Minimum Example

```rust
use std::{env, str::FromStr};

use anyhow::Result;
use solana_crawler::{
    crawler::{Crawler, IxAccount},
    filters::{IxMinAccountsFilter, IxProgramIdFilter, SuccessfulTxFilter, TxHasProgramId},
};
use solana_sdk::pubkey::Pubkey;

#[tokio::main]
async fn main() -> Result<()> {
    // DeGods CMV1 id
    let candy_machine_pubkey = Pubkey::from_str("9MynErYQ5Qi6obp4YwwdoDmXkZ1hYVtPUqYmJJ3rZ9Kn").unwrap();

    let url = "https://ssc-dao.genesysgo.net/"
    let client = solana_client::rpc_client::RpcClient::new(url);

    let has_program_id = TxHasProgramId::new("cndyAnrLdpjq1Ssp1z8xxDsB8dxe7u4HL5Nxi2K5WXZ");
    let successful_tx = SuccessfulTxFilter;
    let ix_program_id = IxProgramIdFilter::new("cndyAnrLdpjq1Ssp1z8xxDsB8dxe7u4HL5Nxi2K5WXZ");
    let ix_min_accounts = IxMinAccountsFilter::new(14);
    let mint_account = IxAccount::new("mint_account", 5);

    let mut crawler = Crawler::new(client, candy_machine_pubkey);
    crawler
        .add_tx_filter(has_program_id)
        .add_tx_filter(successful_tx)
        .add_ix_filter(ix_program_id)
        .add_ix_filter(ix_min_accounts)
        .add_account_index(mint_account);

    let crawled_accounts = crawler.run().await?;

    println!("mint length: {:?}", crawled_accounts["mint_account"].len());
    let f = std::fs::File::create("mint_accounts.json")?;
    serde_json::to_writer(f, &crawled_accounts["mint_account"])?;


    Ok(())
}
```
