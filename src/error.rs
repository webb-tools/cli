use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Subxt(#[from] subxt::Error),
    #[error("Mnemonic: {}", _0)]
    Mnemonic(String),
    #[error("Secret: {:?}", _0)]
    SecretString(subxt::sp_core::crypto::SecretStringError),
    #[error("Bad Ss58: {:?}", _0)]
    Public(subxt::sp_core::crypto::PublicError),
}
