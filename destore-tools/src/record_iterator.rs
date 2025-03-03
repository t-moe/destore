use crate::flash_utils::FlashVec;
use anyhow::{anyhow, bail};
use futures::executor::block_on;
use log::info;
use postcard_dyn::from_slice_dyn;
use postcard_schema::key::hash::fnv1a64_owned::hash_ty_path_owned;
use postcard_schema::schema::owned::OwnedDataModelType;
use sequential_storage::cache::NoCache;
use std::ops::Deref;

const ID_SCHEMA: u8 = 0xFF;

pub fn unpack_partition(partition: &mut [u8], schema: OwnedDataModelType) -> anyhow::Result<()> {
    let schema_hash = hash_ty_path_owned("", &schema);

    info!(
        "Unpacking partition with schema {:#x?}: {:#?}",
        schema_hash, schema
    );
    info!("partition size: {}", partition.len());

    let range = 0..partition.len() as u32;
    let mut flash = FlashVec(partition);
    let mut cache = NoCache::new();
    let mut it = block_on(sequential_storage::queue::iter(
        &mut flash, range, &mut cache,
    ))?;
    let mut buf = [0; 1024];

    loop {
        if let Some(entry) = block_on(it.next(&mut buf))
            .map_err(|_| anyhow!("Failed to fetch next batch of logs"))?
        {
            if entry.len() == 0 {
                bail!("Empty entry");
            }
            if entry[0] == ID_SCHEMA {
                info!("Schema entry");
                let hash = &entry[1..];
                if schema_hash == hash {
                    info!("Schema matches");
                } else {
                    bail!("Schema unknown {:#x?}", hash);
                }
            } else {
                let value = from_slice_dyn(&schema, entry.deref())
                    .map_err(|e| anyhow!("Failed to decode entry: {:?}", e))?;
                info!("Data entry: {:?}", value);
            }
        } else {
            break;
        }
    }

    Ok(())
}
