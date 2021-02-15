use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub struct AccountRaw {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(string, tag = "2")]
    pub alias: String,
    #[prost(string, tag = "3")]
    pub address: String,
    #[prost(bool, tag = "4")]
    pub is_default: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct AccountsIds {
    #[prost(repeated, string, tag = "1")]
    pub ids: Vec<String>,
}
