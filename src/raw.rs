use std::convert::TryInto;
use std::str::FromStr;

use prost::Message;
use webb_cli::account::Account;

#[derive(Clone, PartialEq, Message)]
pub struct AccountRaw {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(string, tag = "2")]
    pub alias: String,
    #[prost(string, tag = "3")]
    pub address: String,
    #[prost(bytes, tag = "4")]
    pub seed: Vec<u8>,
    #[prost(bool, tag = "5")]
    pub is_default: bool,
}

impl AccountRaw {
    pub fn into_account(self, password: &str) -> Account {
        let uuid = uuid::Uuid::from_str(&self.uuid)
            .expect("Failed to parse account uuid");
        Account::init(
            uuid,
            self.alias,
            self.seed
                .try_into()
                .expect("seed must be at least 32 bytes"),
        )
    }
}
