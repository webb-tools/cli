use codec::{Decode, Encode};
use frame_support::Parameter;
use subxt::balances::*;
use subxt::sp_runtime::traits::AtLeast32Bit;
use subxt::system::*;

use super::*;

#[subxt::module]
pub trait Merkle: System + Balances {
    /// The overarching group ID type
    type GroupId: 'static
        + Encode
        + Decode
        + Parameter
        + AtLeast32Bit
        + Default
        + Copy
        + Send
        + Sync;
}

// Storage ..

#[derive(Clone, Debug, Eq, Encode, PartialEq, subxt::Store)]
pub struct CachedRootsStore<T: Merkle> {
    #[store(returns = Vec<Data>)]
    block_number: T::BlockNumber,
    group_id: T::GroupId,
}

impl<T: Merkle> CachedRootsStore<T> {
    pub fn new(block_number: T::BlockNumber, group_id: T::GroupId) -> Self {
        Self {
            block_number,
            group_id,
        }
    }
}
