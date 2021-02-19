use codec::{Decode, Encode};

pub mod merkle;
pub mod mixer;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Encode, Decode)]
pub struct Data(pub [u8; 32]);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Encode, Decode)]
pub struct Nullifier(pub [u8; 32]);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Encode, Decode)]
pub struct Commitment(pub [u8; 32]);
