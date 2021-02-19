#![allow(clippy::too_many_arguments)]

use std::marker::PhantomData;

use codec::{Decode, Encode};
use frame_support::Parameter;
use subxt::balances::*;
use subxt::sp_runtime::traits::AtLeast32Bit;
use subxt::system::*;

use crate::runtime::BalanceOf;

use super::*;

#[subxt::module]
pub trait Mixer: System + Balances {
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
    type Data: Encode
        + Decode
        + Parameter
        + PartialEq
        + Eq
        + Default
        + Send
        + Sync
        + 'static;
    type Nullifier: Encode
        + Decode
        + Parameter
        + PartialEq
        + Eq
        + Default
        + Send
        + Sync
        + 'static;
    type Commitment: Encode
        + Decode
        + Parameter
        + PartialEq
        + Eq
        + Default
        + Send
        + Sync
        + 'static;
}

// return types ..

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct MixerInfo<T: Mixer> {
    pub minimum_deposit_length_for_reward: T::BlockNumber,
    pub fixed_deposit_size: BalanceOf<T>,
    pub leaves: Vec<Data>,
}

// Storage ..

#[derive(Clone, Debug, Eq, Encode, PartialEq, subxt::Store)]
pub struct MixerGroupsStore<T: Mixer> {
    #[store(returns = MixerInfo<T>)]
    id: T::GroupId,
}

impl<T: Mixer> MixerGroupsStore<T> {
    pub fn new(id: T::GroupId) -> Self { Self { id } }
}

#[derive(Clone, Debug, Eq, Encode, PartialEq, subxt::Store)]
pub struct MixerGroupIdsStore<T: Mixer> {
    #[store(returns = Vec<T::GroupId>)]
    __unused: PhantomData<T>,
}

impl<T: Mixer> Default for MixerGroupIdsStore<T> {
    fn default() -> Self {
        Self {
            __unused: PhantomData,
        }
    }
}

// Events ..

#[derive(Clone, Debug, Decode, Eq, PartialEq, subxt::Event)]
pub struct DepositEvent<T: Mixer> {
    group_id: T::GroupId,
    account_id: T::AccountId,
    nullifier: Nullifier,
}

#[derive(Clone, Debug, Decode, Eq, PartialEq, subxt::Event)]
pub struct WithdrawEvent<T: Mixer> {
    group_id: T::GroupId,
    account_id: T::AccountId,
    nullifier: Nullifier,
}

// Calls ..

#[derive(Clone, Debug, Encode, Eq, PartialEq, subxt::Call)]
pub struct DepositCall<T: Mixer> {
    group_id: T::GroupId,
    data_points: Vec<Data>,
}

#[derive(Clone, Debug, Encode, Eq, PartialEq, subxt::Call)]
pub struct WithdrawCall<T: Mixer> {
    group_id: T::GroupId,
    cached_block: T::BlockNumber,
    cached_root: Data,
    comms: Vec<Commitment>,
    nullifier_hash: Data,
    proof_bytes: Vec<u8>,
    leaf_index_commitments: Vec<Commitment>,
    proof_commitments: Vec<Commitment>,
}
