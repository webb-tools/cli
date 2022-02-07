use std::fmt;

use console::Emoji;
use prost::Message;
use webb_cli::note::Note;

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
        write!(
            f,
            "{} ",
            if self.is_default {
                Emoji("📌 ", "*")
            } else {
                Emoji("👤 ", "-")
            }
        )?;
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
    #[prost(bool, tag = "3")]
    pub used: bool,
    #[prost(string, tag = "4")]
    pub value: String,
}

impl fmt::Display for NoteRaw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let note = self.value.parse::<Note>().map_err(|e| {
            let _ = write!(f, "Error parsing note: {}", e);
            fmt::Error
        })?;
        write!(
            f,
            "{} ",
            if self.used {
                Emoji("📦 ", "*")
            } else {
                Emoji("✔️ ", "-")
            }
        )?;
        write!(
            f,
            "{} with {} {}",
            self.alias, note.amount, note.token_symbol,
        )?;
        Ok(())
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct NotesIds {
    #[prost(repeated, string, tag = "1")]
    pub ids: Vec<String>,
}
