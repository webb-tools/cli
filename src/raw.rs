use std::fmt;

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

impl fmt::Display for AccountRaw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", if self.is_default { "*" } else { "-" })?;
        write!(f, "{}: {}", self.alias, self.address)?;
        Ok(())
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct AccountsIds {
    #[prost(repeated, string, tag = "1")]
    pub ids: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct NoteRaw {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(string, tag = "2")]
    pub alias: String,
    #[prost(string, tag = "3")]
    token_symbol: String,
    #[prost(uint32, tag = "4")]
    mixer_id: u32,
    #[prost(bool, tag = "6")]
    pub used: bool,
}

impl fmt::Display for NoteRaw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", if self.used { "*" } else { "-" })?;
        write!(f, "{} with {} Token", self.alias, self.token_symbol)?;
        Ok(())
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct NotesIds {
    #[prost(repeated, string, tag = "1")]
    pub ids: Vec<String>,
}
