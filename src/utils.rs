use anyhow::Result;
use bip39::{Language, Mnemonic};
use secrecy::{ExposeSecret, SecretString};

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
