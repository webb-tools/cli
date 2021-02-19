use std::io::Write;

use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use console::style;
use secrecy::{ExposeSecret, SecretString};
use sha2::Digest;

/// Parse a sercret string, returning a displayable error.
pub fn secret_string_from_str(s: &str) -> Result<SecretString> {
    std::str::FromStr::from_str(s).context("read secret string")
}

pub fn ask_for_phrase(prompt: &str) -> Result<Mnemonic> {
    let mut term = console::Term::stdout();
    loop {
        writeln!(term, "{}", style(prompt).bold().yellow())?;
        let mut words = Vec::with_capacity(12);
        while words.len() < 12 {
            let line = term.read_line()?;
            for word in line.split(' ') {
                words.push(word.trim().to_string());
            }
        }
        if let Ok(mnemonic) =
            Mnemonic::from_phrase(&words.join(" "), Language::English)
        {
            return Ok(mnemonic);
        }
        writeln!(term, "Invalid mnemonic")?;
    }
}

pub fn sha256(s: SecretString) -> Vec<u8> {
    let mut hasher = sha2::Sha256::new();
    hasher.update(s.expose_secret());
    hasher.finalize().to_vec()
}
