use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Subxt(#[from] substrate_subxt::Error),
    #[error("Mnemonic: {}", _0)]
    Mnemonic(String),
    #[error("Bad Ss58: {:?}", _0)]
    PublicError(substrate_subxt::sp_core::crypto::PublicError),
}
