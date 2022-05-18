use subxt::{ClientBuilder, DefaultConfig, PolkadotExtrinsicParams};
use log::*;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod runtime {}

pub async fn calc_tps(node: &String, n: usize) -> Result<(), Box<dyn std::error::Error>> {
    let api = ClientBuilder::new()
        .set_url(node)
        .build()
        .await?
        .to_runtime_api::<runtime::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>>();

    let storage_timestamp = api.storage().timestamp();

    let genesis_hash = api
        .client
        .rpc()
        .block_hash(Some((0 as u32).into()))
        .await?
        .unwrap();

    let mut last_block_timestamp = storage_timestamp.now(Some(genesis_hash)).await?;

    let mut block_n: u32 = 1;
    let mut total_count = 0;
    let mut tps_vec = Vec::new();

    loop {
        let block_hash = api
            .client
            .rpc()
            .block_hash(Some(block_n.into()))
            .await?
            .unwrap();

        let block_timestamp = storage_timestamp.now(Some(block_hash)).await?;
        let time_diff = block_timestamp - last_block_timestamp;
        last_block_timestamp = block_timestamp;

        let mut tps_count = 0;
        let events = api.events().at(block_hash).await?;
        for e in events.iter_raw() {
            if let Ok(raw_event) = e {
                if raw_event.pallet == "Balances" && raw_event.variant == "Transfer" {
                    total_count = total_count + 1;
                    tps_count = tps_count + 1;
                }
            }
        }

        if tps_count > 0 {
            let tps = tps_count as f32 / (time_diff as f32 / 1000.0);
            tps_vec.push(tps);
            info!("TPS on block {}: {}", block_n, tps);
        }

        block_n = block_n + 1;
        if total_count >= n {
            let avg_tps: f32 = tps_vec.iter().sum::<f32>() / tps_vec.len() as f32;
            info!("average TPS: {}", avg_tps);
            break;
        }
    }

    Ok(())
}