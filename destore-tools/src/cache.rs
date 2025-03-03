use log::info;
use postcard_schema::key::hash::fnv1a64_owned::hash_ty_path_owned;
use postcard_schema::schema::owned::OwnedDataModelType;
use std::fs;
use std::path::PathBuf;

pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    pub fn new() -> Self {
        let mut dir = PathBuf::from(
            std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"),
        );
        dir.push(".destore");
        info!("Destore Cache Directory: {:?}", dir);
        if !dir.exists() {
            std::fs::create_dir(&dir).expect("failed to create cache dir");
        }
        Self { dir }
    }

    pub fn store(&mut self, schema: &OwnedDataModelType) -> anyhow::Result<()> {
        let schema_hash = hash_ty_path_owned("", schema);
        let schema_hash: String = schema_hash.iter().map(|b| format!("{:02x}", b)).collect();

        let mut path = self.dir.clone();
        path.push(format!("{}.pcs", schema_hash));

        if path.exists() {
            info!("Schema {} already stored", schema_hash);
            return Ok(());
        }
        let encoded_schema = postcard::to_allocvec(schema)?;
        fs::write(&path, &encoded_schema)?;

        info!("Stored schema {} to {:?}", schema_hash, path);
        Ok(())
    }

    pub fn lookup(&mut self, schema_hash: &[u8; 8]) -> anyhow::Result<Option<OwnedDataModelType>> {
        let schema_hash: String = schema_hash.iter().map(|b| format!("{:02x}", b)).collect();
        let mut path = self.dir.clone();
        path.push(format!("{}.pcs", schema_hash));
        if !path.exists() {
            info!(
                "Schema {:?} not found in cache dir {:?}",
                schema_hash, self.dir
            );
            return Ok(None);
        }
        Ok(postcard::from_bytes(&fs::read(&path)?)?)
    }
}
