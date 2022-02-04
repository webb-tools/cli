use thiserror::Error;
use webb::substrate::protocol_substrate_runtime::api::DispatchError;
use webb::substrate::subxt;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Subxt(#[from] subxt::Error<DispatchError>),
    #[error("Mnemonic: {}", _0)]
    Mnemonic(String),
    #[error("Secret: {:?}", _0)]
    SecretString(subxt::sp_core::crypto::SecretStringError),
    #[error("Bad Ss58: {:?}", _0)]
    Public(subxt::sp_core::crypto::PublicError),
    #[error(transparent)]
    Hex(#[from] hex::FromHexError),
    #[error("Unsupported Token Symbol: {}", _0)]
    UnsupportedTokenSymbol(String),
    #[error("Unsupported Note Version: {}", _0)]
    UnsupportedNoteVersion(String),
    #[error("Unsupported Note Backend: {}", _0)]
    UnsupportedNoteBackend(String),
    #[error("Unsupported Note Curve: {}", _0)]
    UnsupportedNoteCurve(String),
    #[error("Unsupported Note Hash Function: {}", _0)]
    UnsupportedNoteHashFunction(String),
    #[error("Unsupported Note Prefix: {}", _0)]
    UnsupportedNotePrefix(String),
    #[error("Invalid Note format! Please double check your note.")]
    InvalidNoteFormat,
    #[error("Invalid Chain Id in your note.")]
    InvalidChainId,
    #[error("Invalid Note Secrets (must be 64 bytes).")]
    InvalidNoteSecrets,
    #[error("not A 32 bytes array")]
    NotA32BytesArray,
    #[error("Failed to generate secure secrets.")]
    FailedToGenerateSecrets,
}
