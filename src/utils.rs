use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use argon2::{
    password_hash::SaltString, PasswordHash, PasswordHasher, PasswordVerifier,
};
use bip39::{Language, Mnemonic};
use secrecy::{ExposeSecret, SecretString};

/// Parse a sercret string, returning a displayable error.
pub fn secret_string_from_str(s: &str) -> Result<SecretString> {
    std::str::FromStr::from_str(s).context("read secret string")
}

pub fn get_password(
    home: PathBuf,
    secret: Option<SecretString>,
) -> Result<SecretString> {
    let password_hash_file = home.join("passwd_hash");
    if password_hash_file.exists() {
        let contents = fs::read_to_string(password_hash_file)
            .context("read password hash file")?;
        let hash = PasswordHash::new(&contents)
            .map_err(|_| anyhow!("bad password hash file"))?;
        let password = secret
            .or_else(|| ask_for_password("Password: ", 8).ok())
            .ok_or_else(|| anyhow!("no password provided"))?;
        verify_password_hash(&password, hash)?;
        Ok(password)
    } else {
        let password = secret
            .or_else(|| ask_for_new_password(8).ok())
            .ok_or_else(|| anyhow!("no password provided"))?;
        let hashed = hash_password(password.clone())?;
        fs::write(password_hash_file, hashed)?;
        Ok(password)
    }
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

pub fn verify_password_hash(
    password: &SecretString,
    hash: PasswordHash,
) -> Result<()> {
    argon2::Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &hash)
        .is_ok()
        .then(|| ())
        .ok_or_else(|| anyhow!("Password mismatch"))
}

pub fn ask_for_new_password(length: u8) -> Result<SecretString> {
    loop {
        let password = ask_for_password(
            "Please enter a new password (8+ characters): ",
            length,
        )?;
        let password2 =
            ask_for_password("Please confirm your new password: ", length)?;
        if password.expose_secret() == password2.expose_secret() {
            return Ok(password);
        }
        println!("Passwords don't match.");
    }
}

pub fn ask_for_password(prompt: &str, length: u8) -> Result<SecretString> {
    loop {
        let password =
            SecretString::new(rpassword::prompt_password_stdout(prompt)?);
        if password.expose_secret().len() >= length as usize {
            return Ok(password);
        }
        println!(
            "Password too short, needs to be at least {} characters.",
            length
        );
    }
}

pub fn ask_for_phrase(prompt: &str) -> Result<Mnemonic> {
    loop {
        println!("{}", prompt);
        let mut words = Vec::with_capacity(12);
        while words.len() < 12 {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            for word in line.split(' ') {
                words.push(word.trim().to_string());
            }
        }
        println!();
        if let Ok(mnemonic) =
            Mnemonic::from_phrase(&words.join(" "), Language::English)
        {
            println!("{}", mnemonic);
            return Ok(mnemonic);
        }
        println!("Invalid mnemonic");
    }
}

pub fn sha256(bytes: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}
