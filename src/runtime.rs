#![allow(clippy::too_many_arguments)]

use codec::{Decode, Encode};
use frame_support::Parameter;
use subxt::balances::*;
use subxt::extrinsic::*;
use subxt::sp_core;
use subxt::sp_runtime::generic::Header;
use subxt::sp_runtime::traits::{
    AtLeast32Bit, BlakeTwo256, IdentifyAccount, Verify,
};
use subxt::sp_runtime::{MultiSignature, OpaqueExtrinsic};
use subxt::system::*;

/// an easy way to extract the balance type from `T`
pub type BalanceOf<T> = <T as Balances>::Balance;

/// Alias to 512-bit hash when used in the context of a transaction signature on
/// the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it
/// equivalent to the public key of our transaction signing scheme.
pub type AccountId =
    <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of
/// them, but you never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Webb Runtime with `mixer` pallet.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WebbRuntime;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Encode, Decode)]
pub struct Data(pub [u8; 32]);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Encode, Decode)]
pub struct Nullifier(pub [u8; 32]);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Encode, Decode)]
pub struct Commitment(pub [u8; 32]);

impl subxt::Runtime for WebbRuntime {
    type Extra = DefaultExtra<Self>;
    type Signature = Signature;
}

impl System for WebbRuntime {
    type AccountData = AccountData<BalanceOf<Self>>;
    type AccountId = AccountId;
    type Address = AccountId;
    type BlockNumber = u32;
    type Extrinsic = OpaqueExtrinsic;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = Header<Self::BlockNumber, BlakeTwo256>;
    type Index = Index;
}

impl Balances for WebbRuntime {
    type Balance = Balance;
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct MixerInfo<T: Mixer> {
    pub minimum_deposit_length_for_reward: T::BlockNumber,
    pub fixed_deposit_size: BalanceOf<T>,
    pub leaves: Vec<Data>,
}

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

// Storage ..

#[derive(Clone, Debug, Eq, Encode, PartialEq, subxt::Store)]
pub struct MixerGroupsStore<T: Mixer> {
    #[store(returns = MixerInfo<T>)]
    id: T::GroupId,
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

impl Mixer for WebbRuntime {
    type Commitment = Commitment;
    type Data = Data;
    type GroupId = u32;
    type Nullifier = Nullifier;
}

#[cfg(all(test, feature = "integration-tests"))]
mod tests {
    use super::*;
    use sp_keyring::AccountKeyring;
    use subxt::PairSigner;

    type MixerGroups = MixerGroupsStore<WebbRuntime>;

    async fn get_client() -> subxt::Client<WebbRuntime> {
        subxt::ClientBuilder::new()
            .set_url("ws://127.0.0.1:9944")
            .build()
            .await
            .unwrap()
    }

    #[async_std::test]
    async fn get_all_mixers() {
        let client = get_client().await;
        let mut iter = client.iter::<MixerGroups>(None).await.unwrap();
        let mut groups = Vec::new();
        while let Some((_, info)) = iter.next().await.unwrap() {
            groups.push(info);
        }

        assert!(!groups.is_empty());
    }

    #[async_std::test]
    async fn deposit() {
        let client = get_client().await;
        let signer = PairSigner::new(AccountKeyring::Alice.pair());
        let leaf = Data([1u8; 32]);
        let result = client.deposit_and_watch(&signer, 3, vec![leaf]).await;
        assert!(result.is_ok());
        let xt = result.unwrap();
        println!("Hash: {:?}", xt.block);
        let maybe_block = client.block(Some(xt.block)).await.unwrap();
        let signed_block = maybe_block.unwrap();
        println!("Number: #{}", signed_block.block.header.number);
    }
}
