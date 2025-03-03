use crate::flash_utils::FlashVec;
use crate::Cache;
use anyhow::{anyhow, bail};
use futures::executor::block_on;
use log::info;
use postcard_dyn::from_slice_dyn;
use sequential_storage::cache::NoCache;
use std::ops::Deref;

const ID_SCHEMA: u8 = 0xFF;

pub fn unpack_partition(partition: &mut [u8]) -> anyhow::Result<()> {
    let mut schema_cache = Cache::new();
    info!("partition size: {}", partition.len());

    let range = 0..partition.len() as u32;
    let mut flash = FlashVec(partition);
    let mut cache = NoCache::new();
    let mut it = block_on(sequential_storage::queue::iter(
        &mut flash, range, &mut cache,
    ))?;
    let mut buf = [0; 1024];
    let mut schema = None;
    loop {
        if let Some(entry) = block_on(it.next(&mut buf))
            .map_err(|_| anyhow!("Failed to fetch next batch of logs"))?
        {
            if entry.len() == 0 {
                bail!("Empty entry");
            }
            if entry[0] == ID_SCHEMA {
                let hash: &[u8; 8] = &entry[1..].try_into()?;
                info!(
                    "Schema entry: {}",
                    hash.iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>()
                );
                if let Some(s) = schema_cache.lookup(hash)? {
                    schema = Some(s);
                } else {
                    bail!("Schema not found: {:?}", hash);
                }
            } else {
                if let Some(schema) = schema.as_ref() {
                    let value = from_slice_dyn(&schema, entry.deref())
                        .map_err(|e| anyhow!("Failed to decode entry: {:?}", e))?;
                    info!("Data entry: {:?}", value);
                } else {
                    bail!("Cannot decode data entry without schema");
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}
