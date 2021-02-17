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
    #[error(transparent)]
    Hex(#[from] hex::FromHexError),
    #[error("Unsupported Token Symbol: {}", _0)]
    UnsupportedTokenSymbol(String),
    #[error("Unsupported Note Version: {}", _0)]
    UnsupportedNoteVersion(String),
    #[error("Invalid Note Length")]
    InvalidNoteLength,
    #[error("Invalid Note Prefix")]
    InvalidNotePrefix,
    #[error("Invalid Note Mixer ID")]
    InvalidNoteMixerId,
    #[error("Invalid Note Block Number")]
    InvalidNoteBlockNumber,
    #[error("Invalid Note Footer")]
    InvalidNoteFooter,
    #[error("not A 32 bytes array")]
    NotA32BytesArray,
}
