use postcard_schema::Schema;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Schema)]
pub struct Sub {
    pub name: String,
    pub age: u8,
}


#[derive(Serialize, Deserialize, Schema)]
pub enum Record {
    Boot(u8),
    Message(String),
    Sub(Sub)
}