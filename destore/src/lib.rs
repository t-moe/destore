#![no_std]

use core::marker::PhantomData;
use core::ops::Range;
use embedded_storage_async::nor_flash::NorFlash;
use postcard_schema::key::hash::fnv1a64::hash_ty_path;
use sequential_storage::cache::NoCache;
use serde::Serialize;
pub use postcard_schema::schema::DataModelType;
pub use postcard_schema::Schema;


#[macro_export]
macro_rules! export_schema {
    ($val:ty) => {
        #[link_section = ".destore.schema"]
        #[used]
        #[no_mangle] // prevent invoking the macro multiple times
        static _DESTORE_SCHEMA: &'static $crate::DataModelType = <$val as $crate::Schema>::SCHEMA;
    };
    ($id:ident, $val:ty) => {
        #[link_section = ".destore.schema"]
        #[used]
        #[no_mangle] // prevent invoking the macro multiple times
        static $id: &'static $crate::DataModelType = <$val as $crate::Schema>::SCHEMA;
    };
}


pub struct Storer<F: NorFlash, T : Schema + Serialize> {
    flash: F,
    flash_range: Range<u32>,
    phantom_data: PhantomData<T>
}

const ID_SCHEMA: u8 = 0xFF;

impl<F: NorFlash, T: Schema + Serialize> Storer<F,T> {
    pub async fn new(mut flash: F, flash_range: Range<u32>) -> Result<Self, sequential_storage::Error<F::Error>> {

        // Store schema hash
        let hash = hash_ty_path::<T>("");
        let mut bytes = [0;9];
        bytes[0] = ID_SCHEMA;
        bytes[1..9].copy_from_slice(&hash);
        sequential_storage::queue::push(&mut flash, flash_range.clone(), &mut NoCache::new(), &bytes, true).await?;

        Ok(Self {
            flash,
            flash_range,
            phantom_data: PhantomData
        })
    }


    pub async fn write(&mut self, record: &T) -> Result<(), sequential_storage::Error<F::Error>> {

        let bytes = postcard::to_allocvec(record).unwrap();
        sequential_storage::queue::push(&mut self.flash, self.flash_range.clone(), &mut NoCache::new(), &bytes, true).await
    }
}


