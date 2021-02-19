use subxt::balances::*;
use subxt::extrinsic::*;
use subxt::sp_core;
use subxt::sp_runtime::generic::Header;
use subxt::sp_runtime::traits::{BlakeTwo256, IdentifyAccount, Verify};
use subxt::sp_runtime::{MultiSignature, OpaqueExtrinsic};
use subxt::system::*;

use crate::pallet;

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

impl pallet::mixer::Mixer for WebbRuntime {
    type Commitment = pallet::Commitment;
    type Data = pallet::Data;
    type GroupId = u32;
    type Nullifier = pallet::Nullifier;
}

impl pallet::merkle::Merkle for WebbRuntime {
    type GroupId = u32;
}

#[cfg(all(test, feature = "integration-tests"))]
mod tests {
    use super::*;
    use crate::pallet::merkle::*;
    use crate::pallet::mixer::*;
    use crate::pallet::Data;
    use sp_keyring::AccountKeyring;
    use subxt::PairSigner;

    type MixerGroups = MixerGroupsStore<WebbRuntime>;
    type MixerGroupIds = MixerGroupIdsStore<WebbRuntime>;
    type CachedRoots = CachedRootsStore<WebbRuntime>;

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

        let ids = client
            .fetch_or_default::<MixerGroupIds>(&MixerGroupIds::default(), None)
            .await
            .unwrap();
        assert!(!ids.is_empty());
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

        let cached_roots = client
            .fetch(&CachedRoots::new(signed_block.block.header.number, 3), None)
            .await
            .unwrap();
        println!("Cached Roots: {:?}", cached_roots);
    }
}
