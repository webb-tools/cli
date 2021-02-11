use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Subxt(#[from] subxt::Error),
    #[error("Mnemonic: {}", _0)]
    Mnemonic(String),
    #[error("Bad Ss58: {:?}", _0)]
    Public(subxt::sp_core::crypto::PublicError),
}
