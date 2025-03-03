use alloc::string::String;
use postcard_schema::Schema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Schema)]
pub struct Sub {
    pub first_name: String,
    pub last_name: String,
    pub age: u8,
    pub brothers: u16,
}

#[derive(Serialize, Deserialize, Schema)]
pub enum Record {
    Boot(u8),
    Message(String),
    Sub(Sub),
    Panic(String),
}
