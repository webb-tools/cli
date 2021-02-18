use std::io::Write;

use anyhow::{anyhow, Context, Result};
use argon2::password_hash::SaltString;
use argon2::PasswordHasher;
use bip39::{Language, Mnemonic};
use console::style;
use secrecy::{ExposeSecret, SecretString};

/// Parse a sercret string, returning a displayable error.
pub fn secret_string_from_str(s: &str) -> Result<SecretString> {
    std::str::FromStr::from_str(s).context("read secret string")
}

pub fn hash_password(password: SecretString) -> Result<String> {
    let mut rng = rand::thread_rng();
    let salt = SaltString::generate(&mut rng);
    argon2::Argon2::default()
        .hash_password_simple(
            password.expose_secret().as_bytes(),
            salt.as_ref(),
        )
        .map_err(|_| anyhow!("Failed to hash the password"))
        .map(|v| v.to_string())
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
