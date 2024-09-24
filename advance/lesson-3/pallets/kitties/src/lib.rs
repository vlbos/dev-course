#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::pallet_macros::import_section;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[path = "ocw_tests.rs"]
// #[cfg(test)]
// mod ocw_tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;
extern crate alloc;
mod config;
mod errors;
mod events;
mod extrinsics;
mod genesis;
mod hooks;
mod impls;
pub mod migration;
mod validate;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_system::{
    offchain::{
        AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
        SignedPayload, Signer, SigningTypes, SubmitTransaction,
    },
    pallet_prelude::BlockNumberFor,
};
use lite_json::json::JsonValue;
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    offchain::{
        http,
        storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
        Duration,
    },
    traits::Zero,
    transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
    RuntimeDebug,
};
/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"dot!");

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KEY_TYPE);

    pub struct TestAuthId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    // implemented for mock runtime in test
    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for TestAuthId
    {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

/// Import all sections from different files.
#[import_section(extrinsics::dispatches)]
#[import_section(errors::errors)]
#[import_section(events::events)]
#[import_section(config::config)]
#[import_section(hooks::hooks)]
#[import_section(impls::impls)]
#[import_section(genesis::genesis)]
#[import_section(validate::validate)]
/// Set the pallet at dev mode for quick PoC. (dev_mode)
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, Randomness, ReservableCurrency},
    };
    use frame_system::{self as system, pallet_prelude::*};
    use sp_runtime::TryRuntimeError;
    use sp_std::prelude::*;
    use sp_weights::WeightMeter;
    // Version 0
    // #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    // pub struct Kitty(pub [u8; 16]);

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    #[codec(mel_bound())]
    pub struct Kitty<T: Config> {
        pub dna: [u8; 16],
        pub price: Option<BalanceOf<T>>,
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    /// The current storage version, we set to 1 our new version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);
    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_kitty_id)]
    pub type NextKittyId<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    // pub type Kitties<T> = StorageMap<_, Blake2_128Concat, u32, Kitty>;//Version 0
    pub type Kitties<T> = StorageMap<_, Blake2_128Concat, u32, Kitty<T>>;

    #[pallet::storage]
    #[pallet::getter(fn kitty_owner)]
    pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, u32, T::AccountId>;

    #[pallet::storage]
    #[pallet::getter(fn kitties_on_sale)]
    pub type KittiesOnSale<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<u32, T::MaxKittiesBidPerBlock>,
        ValueQuery,
    >;

    // bid price for each kitty,
    #[pallet::storage]
    #[pallet::getter(fn kitties_bid)]
    pub type KittiesBid<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, Option<(T::AccountId, BalanceOf<T>)>, ValueQuery>;

    /// A vector of recently submitted prices.
    ///
    /// This is used to calculate average price, should have bounded size.
    #[pallet::storage]
    pub(super) type Prices<T: Config> = StorageValue<_, BoundedVec<u32, T::MaxPrices>, ValueQuery>;

    /// Defines the block when next unsigned transaction will be accepted.
    ///
    /// To prevent spam of unsigned (and unpaid!) transactions on the network,
    /// we only allow one transaction every `T::UnsignedInterval` blocks.
    /// This storage entry defines when new transaction is going to be accepted.
    #[pallet::storage]
    pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;
}

/// Payload used by this example crate to hold price
/// data required to submit a transaction.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
pub struct PricePayload<Public, BlockNumber> {
    block_number: BlockNumber,
    price: u32,
    public: Public,
}

impl<T: SigningTypes> SignedPayload<T> for PricePayload<T::Public, BlockNumberFor<T>> {
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}

enum TransactionType {
    Signed,
    UnsignedForAny,
    UnsignedForAll,
    Raw,
    None,
}
