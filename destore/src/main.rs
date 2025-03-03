use postcard_schema::Schema;
use log::info;
use postcard_dyn::from_slice_dyn;
use postcard_schema::schema::owned::{OwnedNamedType};

mod schema;
use crate::schema::*;


fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let r1 = Record::Sub(Sub { name: "Alice".to_string(), age: 20 });

    let bytes = postcard::to_stdvec(&r1).unwrap();
    let schema_bytes = postcard::to_stdvec(&Record::SCHEMA).unwrap();

    let restored_schema : OwnedNamedType = postcard::from_bytes(&schema_bytes).unwrap();
    let restored = from_slice_dyn(&restored_schema, &bytes).unwrap();


    info!("Schema {:?}", restored_schema);
    info!("schema bytes {:?}", schema_bytes);
    info!("bytes {:?}", bytes);
    info!("restored {:?}", restored);
}
