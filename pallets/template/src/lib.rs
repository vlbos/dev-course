//! # Template Pallet
//!
//! A pallet with minimal functionality to help developers understand the essential components of
//! writing a FRAME pallet. It is typically used in beginner tutorials or in Substrate template
//! nodes as a starting point for creating a new pallet and **not meant to be used in production**.
//!
//! ## Overview
//!
//! This template pallet contains basic examples of:
//! - declaring a storage item that stores a single `u32` value
//! - declaring and using events
//! - declaring and using errors
//! - a dispatchable function that allows a user to set a new value to storage and emits an event
//!   upon success
//! - another dispatchable function that causes a custom error to be thrown
//!
//! Each pallet section is annotated with an attribute using the `#[pallet::...]` procedural macro.
//! This macro generates the necessary code for a pallet to be aggregated into a FRAME runtime.
//!
//! Learn more about FRAME macros [here](https://docs.substrate.io/reference/frame-macros/).
//!
//! ### Pallet Sections
//!
//! The pallet sections in this template are:
//!
//! - A **configuration trait** that defines the types and parameters which the pallet depends on
//!   (denoted by the `#[pallet::config]` attribute). See: [`Config`].
//! - A **means to store pallet-specific data** (denoted by the `#[pallet::storage]` attribute).
//!   See: [`storage_types`].
//! - A **declaration of the events** this pallet emits (denoted by the `#[pallet::event]`
//!   attribute). See: [`Event`].
//! - A **declaration of the errors** that this pallet can throw (denoted by the `#[pallet::error]`
//!   attribute). See: [`Error`].
//! - A **set of dispatchable functions** that define the pallet's functionality (denoted by the
//!   `#[pallet::call]` attribute). See: [`dispatchables`].
//!
//! Run `cargo doc --package pallet-template --open` to view this pallet's documentation.

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use alloc::{vec, vec::Vec};
use codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_system::{
    self as system,
    offchain::{
        AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
        SignedPayload, Signer, SigningTypes, SubmitTransaction,
    },
    pallet_prelude::BlockNumberFor,
};
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
// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
#[cfg(test)]
mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");

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

    // implemented for runtime
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

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
    // Import various useful types required by all FRAME pallets.
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::offchain::SendSignedTransaction;
    use frame_system::pallet_prelude::*;

    // The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
    // (`Call`s) in this pallet.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The pallet's configuration trait.
    ///
    /// All our types and constants a pallet depends on must be declared here.
    /// These types are defined generically and made concrete when the pallet is declared in the
    /// `runtime/src/lib.rs` file of your chain.
    #[pallet::config]
    pub trait Config:
        frame_system::offchain::CreateSignedTransaction<Call<Self>> + frame_system::Config
    {
        type AuthorityId: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>;
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;
    }

    /// A storage item for this pallet.
    ///
    /// In this template, we are declaring a storage item called `Something` that stores a single
    /// `u32` value. Learn more about runtime storage here: <https://docs.substrate.io/build/runtime-storage/>
    #[pallet::storage]
    pub type Something<T> = StorageValue<_, u32>;

    /// Events that functions in this pallet can emit.
    ///
    /// Events are a simple means of indicating to the outside world (such as dApps, chain explorers
    /// or other users) that some notable update in the runtime has occurred. In a FRAME pallet, the
    /// documentation for each event field and its parameters is added to a node's metadata so it
    /// can be used by external interfaces or tools.
    ///
    ///	The `generate_deposit` macro generates a function on `Pallet` called `deposit_event` which
    /// will convert the event type of your pallet into `RuntimeEvent` (declared in the pallet's
    /// [`Config`] trait) and deposit it using [`frame_system::Pallet::deposit_event`].
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A user has successfully set a new value.
        SomethingStored {
            /// The new value set.
            something: u32,
            /// The account who set the new value.
            who: T::AccountId,
        },
    }

    /// Errors that can be returned by this pallet.
    ///
    /// Errors tell users that something went wrong so it's important that their naming is
    /// informative. Similar to events, error documentation is added to a node's metadata so it's
    /// equally important that they have helpful documentation associated with them.
    ///
    /// This type of runtime error can be up to 4 bytes in size should you want to return additional
    /// information.
    #[pallet::error]
    pub enum Error<T> {
        /// The value retrieved was `None` as no value was previously set.
        NoneValue,
        /// There was an attempt to increment the value in storage over `u32::MAX`.
        StorageOverflow,
    }
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Offchain worker entry point.
        ///
        /// By implementing `fn offchain_worker` you declare a new offchain worker.
        /// This function will be called when the node is fully synced and a new best block is
        /// successfully imported.
        /// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
        /// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
        /// so the code should be able to handle that.
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            log::info!("Hello from pallet-ocw.");
            // The entry point of your code called by offchain worker
            let one = BlockNumberFor::<T>::from(1u32);
            let zero = BlockNumberFor::<T>::zero();
            let _ = match block_number % 3u32.into() {
                b if b == zero => Self::send_signed_tx(vec![0, 1, 2, 3]),
                b if b == one => Self::send_unsigned_tx(42),
                _ => Self::send_unsigned_tx_with_payload(1),
            };
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        /// Validate unsigned call to this module.
        ///
        /// By default unsigned transactions are disallowed, but implementing the validator
        /// here we make sure that some particular calls (the ones produced by offchain worker)
        /// are being whitelisted and marked as valid.
        fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            const UNSIGNED_TXS_PRIORITY: u64 = 100;
            let valid_tx = |provide| {
                ValidTransaction::with_tag_prefix("my-pallet")
                    .priority(UNSIGNED_TXS_PRIORITY) // please define `UNSIGNED_TXS_PRIORITY` before this line
                    .and_provides([&provide])
                    .longevity(3)
                    .propagate(true)
                    .build()
            };

            match call {
                Call::unsigned_extrinsic_with_signed_payload {
                    ref payload,
                    ref signature,
                } => {
                    if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
                        return InvalidTransaction::BadProof.into();
                    }
                    valid_tx(b"unsigned_extrinsic_with_signed_payload".to_vec())
                }
                Call::submit_data_unsigned { key: _ } => valid_tx(b"my_unsigned_tx".to_vec()),
                _ => InvalidTransaction::Call.into(),
            }
        }
    }
    /// The pallet's dispatchable functions ([`Call`]s).
    ///
    /// Dispatchable functions allows users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// They must always return a `DispatchResult` and be annotated with a weight and call index.
    ///
    /// The [`call_index`] macro is used to explicitly
    /// define an index for calls in the [`Call`] enum. This is useful for pallets that may
    /// introduce new dispatchables over time. If the order of a dispatchable changes, its index
    /// will also change which will break backwards compatibility.
    ///
    /// The [`weight`] macro is used to assign a weight to each call.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a single u32 value as a parameter, writes the value
        /// to storage and emits an event.
        ///
        /// It checks that the _origin_ for this call is _Signed_ and returns a dispatch
        /// error if it isn't. Learn more about origins here: <https://docs.substrate.io/build/origins/>
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            // Update storage.
            Something::<T>::put(something);

            // Emit an event.
            Self::deposit_event(Event::SomethingStored { something, who });

            // Return a successful `DispatchResult`
            Ok(())
        }

        /// An example dispatchable that may throw a custom error.
        ///
        /// It checks that the caller is a signed origin and reads the current value from the
        /// `Something` storage item. If a current value exists, it is incremented by 1 and then
        /// written back to storage.
        ///
        /// ## Errors
        ///
        /// The function will return an error under the following conditions:
        ///
        /// - If no value has been set ([`Error::NoneValue`])
        /// - If incrementing the value in storage causes an arithmetic overflow
        ///   ([`Error::StorageOverflow`])
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::cause_error())]
        pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Read a value from storage.
            match Something::<T>::get() {
                // Return an error if the value has not been set.
                None => Err(Error::<T>::NoneValue.into()),
                Some(old) => {
                    // Increment the value read from storage. This will cause an error in the event
                    // of overflow.
                    let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
                    // Update the value in storage with the incremented result.
                    Something::<T>::put(new);
                    Ok(())
                }
            }
        }

        #[pallet::call_index(2)]
        #[pallet::weight({0})]
        pub fn submit_data(origin: OriginFor<T>, payload: Vec<u8>) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            log::info!("OCW ==> in submit_data call: {:?}", payload);
            Ok(().into())
        }
        #[pallet::call_index(3)]
        #[pallet::weight({0})]
        pub fn submit_data_unsigned(origin: OriginFor<T>, key: u64) -> DispatchResult {
            ensure_none(origin)?;

            log::info!("OCW ==> in submit_data_unsigned: {:?}", key);
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }
        #[pallet::call_index(4)]
        #[pallet::weight({0})]
        pub fn unsigned_extrinsic_with_signed_payload(
            origin: OriginFor<T>,
            payload: Payload<T::Public>,
            _signature: T::Signature,
        ) -> DispatchResult {
            ensure_none(origin)?;

            log::info!(
                "OCW ==> in call unsigned_extrinsic_with_signed_payload: {:?}",
                payload.number
            );
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
pub struct Payload<Public> {
    number: u64,
    public: Public,
}

impl<T: frame_system::offchain::SigningTypes> frame_system::offchain::SignedPayload<T>
    for Payload<T::Public>
{
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}

impl<T: Config> Pallet<T> {
    fn send_signed_tx(payload: Vec<u8>) -> Result<(), &'static str> {
        let signer = Signer::<T, T::AuthorityId>::all_accounts();
        if !signer.can_sign() {
            return Err(
                "No local accounts available. Consider adding one via `author_insertKey` RPC.",
            );
        }

        let results = signer.send_signed_transaction(|_account| Call::submit_data {
            payload: payload.clone(),
        });

        for (acc, res) in &results {
            match res {
                Ok(()) => log::info!("[{:?}] Submitted data:{:?}", acc.id, payload),
                Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
            }
        }

        Ok(())
    }
    fn send_unsigned_tx(value: u64) -> Result<(), &'static str> {
        // let value: u64 = 42;
        // This is your call to on-chain extrinsic together with any necessary parameters.
        let call = Call::submit_data_unsigned { key: value };

        // `submit_unsigned_transaction` returns a type of `Result<(), ()>`
        //	 ref: https://paritytech.github.io/substrate/master/frame_system/offchain/struct.SubmitTransaction.html
        _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).map_err(
            |_| {
                log::error!("OCW ==> Failed in offchain_unsigned_tx");
            },
        );
        Ok(())
    }
    fn send_unsigned_tx_with_payload(number: u64) -> Result<(), &'static str> {
        // let number: u64 = 42;
        // Retrieve the signer to sign the payload
        let signer = Signer::<T, T::AuthorityId>::any_account();

        // `send_unsigned_transaction` is returning a type of `Option<(Account<T>, Result<(), ()>)>`.
        //	 The returned result means:
        //	 - `None`: no account is available for sending transaction
        //	 - `Some((account, Ok(())))`: transaction is successfully sent
        //	 - `Some((account, Err(())))`: error occurred when sending the transaction
        if let Some((_, res)) = signer.send_unsigned_transaction(
            // this line is to prepare and return payload
            |acct| Payload {
                number,
                public: acct.public.clone(),
            },
            |payload, signature| Call::unsigned_extrinsic_with_signed_payload {
                payload,
                signature,
            },
        ) {
            match res {
                Ok(()) => {
                    log::info!("OCW ==> unsigned tx with signed payload successfully sent.");
                }
                Err(()) => {
                    log::error!("OCW ==> sending unsigned tx with signed payload failed.");
                }
            };
        } else {
            // The case of `None`: no account is available for sending
            log::error!("OCW ==> No local account available");
        }
        Ok(())
    }

    fn send_unsigned_tx_with_payload_for_all_accounts(number: u64) -> Result<(), &'static str> {
        // let number: u64 = 42;
        // Retrieve the signer to sign the payload
        let signer = Signer::<T, T::AuthorityId>::all_accounts();

        // `send_unsigned_transaction` is returning a type of `Option<(Account<T>, Result<(), ()>)>`.
        //	 The returned result means:
        //	 - `None`: no account is available for sending transaction
        //	 - `Some((account, Ok(())))`: transaction is successfully sent
        //	 - `Some((account, Err(())))`: error occurred when sending the transaction
        let transaction_results = signer.send_unsigned_transaction(
            // this line is to prepare and return payload
            |acct| Payload {
                number,
                public: acct.public.clone(),
            },
            |payload, signature| Call::unsigned_extrinsic_with_signed_payload {
                payload,
                signature,
            },
        );
        if transaction_results
            .into_iter()
            .any(|(_, result)| result.is_err())
        {
            return Err("Unable to submit transaction");
        }
        Ok(())
    }

    // /// Chooses which transaction type to send.
    // ///
    // /// This function serves mostly to showcase `StorageValue` helper
    // /// and local storage usage.
    // ///
    // /// Returns a type of transaction that should be produced in current run.
    // fn choose_transaction_type(block_number: BlockNumberFor<T>) -> TransactionType {
    // 	/// A friendlier name for the error that is going to be returned in case we are in the grace
    // 	/// period.
    // 	const RECENTLY_SENT: () = ();

    // 	// Start off by creating a reference to Local Storage value.
    // 	// Since the local storage is common for all offchain workers, it's a good practice
    // 	// to prepend your entry with the module name.
    // 	let val = StorageValueRef::persistent(b"example_ocw::last_send");
    // 	// The Local Storage is persisted and shared between runs of the offchain workers,
    // 	// and offchain workers may run concurrently. We can use the `mutate` function, to
    // 	// write a storage entry in an atomic fashion. Under the hood it uses `compare_and_set`
    // 	// low-level method of local storage API, which means that only one worker
    // 	// will be able to "acquire a lock" and send a transaction if multiple workers
    // 	// happen to be executed concurrently.
    // 	let res =
    // 		val.mutate(|last_send: Result<Option<BlockNumberFor<T>>, StorageRetrievalError>| {
    // 			match last_send {
    // 				// If we already have a value in storage and the block number is recent enough
    // 				// we avoid sending another transaction at this time.
    // 				Ok(Some(block)) if block_number < block + T::GracePeriod::get() =>
    // 					Err(RECENTLY_SENT),
    // 				// In every other case we attempt to acquire the lock and send a transaction.
    // 				_ => Ok(block_number),
    // 			}
    // 		});

    // 	// The result of `mutate` call will give us a nested `Result` type.
    // 	// The first one matches the return of the closure passed to `mutate`, i.e.
    // 	// if we return `Err` from the closure, we get an `Err` here.
    // 	// In case we return `Ok`, here we will have another (inner) `Result` that indicates
    // 	// if the value has been set to the storage correctly - i.e. if it wasn't
    // 	// written to in the meantime.
    // 	match res {
    // 		// The value has been set correctly, which means we can safely send a transaction now.
    // 		Ok(block_number) => {
    // 			// We will send different transactions based on a random number.
    // 			// Note that this logic doesn't really guarantee that the transactions will be sent
    // 			// in an alternating fashion (i.e. fairly distributed). Depending on the execution
    // 			// order and lock acquisition, we may end up for instance sending two `Signed`
    // 			// transactions in a row. If a strict order is desired, it's better to use
    // 			// the storage entry for that. (for instance store both block number and a flag
    // 			// indicating the type of next transaction to send).
    // 			let transaction_type = block_number % 4u32.into();
    // 			if transaction_type == Zero::zero() {
    // 				TransactionType::Signed
    // 			} else if transaction_type == BlockNumberFor::<T>::from(1u32) {
    // 				TransactionType::UnsignedForAny
    // 			} else if transaction_type == BlockNumberFor::<T>::from(2u32) {
    // 				TransactionType::UnsignedForAll
    // 			} else {
    // 				TransactionType::Raw
    // 			}
    // 		},
    // 		// We are in the grace period, we should not send a transaction this time.
    // 		Err(MutateStorageError::ValueFunctionFailed(RECENTLY_SENT)) => TransactionType::None,
    // 		// We wanted to send a transaction, but failed to write the block number (acquire a
    // 		// lock). This indicates that another offchain worker that was running concurrently
    // 		// most likely executed the same logic and succeeded at writing to storage.
    // 		// Thus we don't really want to send the transaction, knowing that the other run
    // 		// already did.
    // 		Err(MutateStorageError::ConcurrentModification(_)) => TransactionType::None,
    // 	}
    // }

    // /// A helper function to fetch the price and send signed transaction.
    // fn fetch_price_and_send_signed() -> Result<(), &'static str> {
    // 	let signer = Signer::<T, T::AuthorityId>::all_accounts();
    // 	if !signer.can_sign() {
    // 		return Err(
    // 			"No local accounts available. Consider adding one via `author_insertKey` RPC.",
    // 		)
    // 	}
    // 	// Make an external HTTP request to fetch the current price.
    // 	// Note this call will block until response is received.
    // 	let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

    // 	// Using `send_signed_transaction` associated type we create and submit a transaction
    // 	// representing the call, we've just created.
    // 	// Submit signed will return a vector of results for all accounts that were found in the
    // 	// local keystore with expected `KEY_TYPE`.
    // 	let results = signer.send_signed_transaction(|_account| {
    // 		// Received price is wrapped into a call to `submit_price` public function of this
    // 		// pallet. This means that the transaction, when executed, will simply call that
    // 		// function passing `price` as an argument.
    // 		Call::submit_price { price }
    // 	});

    // 	for (acc, res) in &results {
    // 		match res {
    // 			Ok(()) => log::info!("[{:?}] Submitted price of {} cents", acc.id, price),
    // 			Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
    // 		}
    // 	}

    // 	Ok(())
    // }

    // /// A helper function to fetch the price and send a raw unsigned transaction.
    // fn fetch_price_and_send_raw_unsigned(
    // 	block_number: BlockNumberFor<T>,
    // ) -> Result<(), &'static str> {
    // 	// Make sure we don't fetch the price if unsigned transaction is going to be rejected
    // 	// anyway.
    // 	let next_unsigned_at = NextUnsignedAt::<T>::get();
    // 	if next_unsigned_at > block_number {
    // 		return Err("Too early to send unsigned transaction")
    // 	}

    // 	// Make an external HTTP request to fetch the current price.
    // 	// Note this call will block until response is received.
    // 	let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

    // 	// Received price is wrapped into a call to `submit_price_unsigned` public function of this
    // 	// pallet. This means that the transaction, when executed, will simply call that function
    // 	// passing `price` as an argument.
    // 	let call = Call::submit_price_unsigned { block_number, price };

    // 	// Now let's create a transaction out of this call and submit it to the pool.
    // 	// Here we showcase two ways to send an unsigned transaction / unsigned payload (raw)
    // 	//
    // 	// By default unsigned transactions are disallowed, so we need to whitelist this case
    // 	// by writing `UnsignedValidator`. Note that it's EXTREMELY important to carefully
    // 	// implement unsigned validation logic, as any mistakes can lead to opening DoS or spam
    // 	// attack vectors. See validation logic docs for more details.
    // 	//
    // 	SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
    // 		.map_err(|()| "Unable to submit unsigned transaction.")?;

    // 	Ok(())
    // }

    // /// A helper function to fetch the price, sign payload and send an unsigned transaction
    // fn fetch_price_and_send_unsigned_for_any_account(
    // 	block_number: BlockNumberFor<T>,
    // ) -> Result<(), &'static str> {
    // 	// Make sure we don't fetch the price if unsigned transaction is going to be rejected
    // 	// anyway.
    // 	let next_unsigned_at = NextUnsignedAt::<T>::get();
    // 	if next_unsigned_at > block_number {
    // 		return Err("Too early to send unsigned transaction")
    // 	}

    // 	// Make an external HTTP request to fetch the current price.
    // 	// Note this call will block until response is received.
    // 	let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

    // 	// -- Sign using any account
    // 	let (_, result) = Signer::<T, T::AuthorityId>::any_account()
    // 		.send_unsigned_transaction(
    // 			|account| PricePayload { price, block_number, public: account.public.clone() },
    // 			|payload, signature| Call::submit_price_unsigned_with_signed_payload {
    // 				price_payload: payload,
    // 				signature,
    // 			},
    // 		)
    // 		.ok_or("No local accounts accounts available.")?;
    // 	result.map_err(|()| "Unable to submit transaction")?;

    // 	Ok(())
    // }

    // /// A helper function to fetch the price, sign payload and send an unsigned transaction
    // fn fetch_price_and_send_unsigned_for_all_accounts(
    // 	block_number: BlockNumberFor<T>,
    // ) -> Result<(), &'static str> {
    // 	// Make sure we don't fetch the price if unsigned transaction is going to be rejected
    // 	// anyway.
    // 	let next_unsigned_at = NextUnsignedAt::<T>::get();
    // 	if next_unsigned_at > block_number {
    // 		return Err("Too early to send unsigned transaction")
    // 	}

    // 	// Make an external HTTP request to fetch the current price.
    // 	// Note this call will block until response is received.
    // 	let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

    // 	// -- Sign using all accounts
    // 	let transaction_results = Signer::<T, T::AuthorityId>::all_accounts()
    // 		.send_unsigned_transaction(
    // 			|account| PricePayload { price, block_number, public: account.public.clone() },
    // 			|payload, signature| Call::submit_price_unsigned_with_signed_payload {
    // 				price_payload: payload,
    // 				signature,
    // 			},
    // 		);
    // 	for (_account_id, result) in transaction_results.into_iter() {
    // 		if result.is_err() {
    // 			return Err("Unable to submit transaction")
    // 		}
    // 	}

    // 	Ok(())
    // }

    // /// Fetch current price and return the result in cents.
    // fn fetch_price() -> Result<u32, http::Error> {
    // 	// We want to keep the offchain worker execution time reasonable, so we set a hard-coded
    // 	// deadline to 2s to complete the external call.
    // 	// You can also wait indefinitely for the response, however you may still get a timeout
    // 	// coming from the host machine.
    // 	let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
    // 	// Initiate an external HTTP GET request.
    // 	// This is using high-level wrappers from `sp_runtime`, for the low-level calls that
    // 	// you can find in `sp_io`. The API is trying to be similar to `request`, but
    // 	// since we are running in a custom WASM execution environment we can't simply
    // 	// import the library here.
    // 	let request =
    // 		http::Request::get("https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD");
    // 	// We set the deadline for sending of the request, note that awaiting response can
    // 	// have a separate deadline. Next we send the request, before that it's also possible
    // 	// to alter request headers or stream body content in case of non-GET requests.
    // 	let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;

    // 	// The request is already being processed by the host, we are free to do anything
    // 	// else in the worker (we can send multiple concurrent requests too).
    // 	// At some point however we probably want to check the response though,
    // 	// so we can block current thread and wait for it to finish.
    // 	// Note that since the request is being driven by the host, we don't have to wait
    // 	// for the request to have it complete, we will just not read the response.
    // 	let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
    // 	// Let's check the status code before we proceed to reading the response.
    // 	if response.code != 200 {
    // 		log::warn!("Unexpected status code: {}", response.code);
    // 		return Err(http::Error::Unknown)
    // 	}

    // 	// Next we want to fully read the response body and collect it to a vector of bytes.
    // 	// Note that the return object allows you to read the body in chunks as well
    // 	// with a way to control the deadline.
    // 	let body = response.body().collect::<Vec<u8>>();

    // 	// Create a str slice from the body.
    // 	let body_str = alloc::str::from_utf8(&body).map_err(|_| {
    // 		log::warn!("No UTF8 body");
    // 		http::Error::Unknown
    // 	})?;

    // 	let price = match Self::parse_price(body_str) {
    // 		Some(price) => Ok(price),
    // 		None => {
    // 			log::warn!("Unable to extract price from the response: {:?}", body_str);
    // 			Err(http::Error::Unknown)
    // 		},
    // 	}?;

    // 	log::warn!("Got price: {} cents", price);

    // 	Ok(price)
    // }

    // /// Parse the price from the given JSON string using `lite-json`.
    // ///
    // /// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
    // fn parse_price(price_str: &str) -> Option<u32> {
    // 	let val = lite_json::parse_json(price_str);
    // 	let price = match val.ok()? {
    // 		JsonValue::Object(obj) => {
    // 			let (_, v) = obj.into_iter().find(|(k, _)| k.iter().copied().eq("USD".chars()))?;
    // 			match v {
    // 				JsonValue::Number(number) => number,
    // 				_ => return None,
    // 			}
    // 		},
    // 		_ => return None,
    // 	};

    // 	let exp = price.fraction_length.saturating_sub(2);
    // 	Some(price.integer as u32 * 100 + (price.fraction / 10_u64.pow(exp)) as u32)
    // }

    // /// Add new price to the list.
    // fn add_price(maybe_who: Option<T::AccountId>, price: u32) {
    // 	log::info!("Adding to the average: {}", price);
    // 	<Prices<T>>::mutate(|prices| {
    // 		if prices.try_push(price).is_err() {
    // 			prices[(price % T::MaxPrices::get()) as usize] = price;
    // 		}
    // 	});

    // 	let average = Self::average_price()
    // 		.expect("The average is not empty, because it was just mutated; qed");
    // 	log::info!("Current average price is: {}", average);
    // 	// here we are raising the NewPrice event
    // 	Self::deposit_event(Event::NewPrice { price, maybe_who });
    // }

    // /// Calculate current average price.
    // fn average_price() -> Option<u32> {
    // 	let prices = Prices::<T>::get();
    // 	if prices.is_empty() {
    // 		None
    // 	} else {
    // 		Some(prices.iter().fold(0_u32, |a, b| a.saturating_add(*b)) / prices.len() as u32)
    // 	}
    // }

    // fn validate_transaction_parameters(
    // 	block_number: &BlockNumberFor<T>,
    // 	new_price: &u32,
    // ) -> TransactionValidity {
    // 	// Now let's check if the transaction has any chance to succeed.
    // 	let next_unsigned_at = NextUnsignedAt::<T>::get();
    // 	if &next_unsigned_at > block_number {
    // 		return InvalidTransaction::Stale.into()
    // 	}
    // 	// Let's make sure to reject transactions from the future.
    // 	let current_block = <system::Pallet<T>>::block_number();
    // 	if &current_block < block_number {
    // 		return InvalidTransaction::Future.into()
    // 	}

    // 	// We prioritize transactions that are more far away from current average.
    // 	//
    // 	// Note this doesn't make much sense when building an actual oracle, but this example
    // 	// is here mostly to show off offchain workers capabilities, not about building an
    // 	// oracle.
    // 	let avg_price = Self::average_price()
    // 		.map(|price| if &price > new_price { price - new_price } else { new_price - price })
    // 		.unwrap_or(0);

    // 	ValidTransaction::with_tag_prefix("ExampleOffchainWorker")
    // 		// We set base priority to 2**20 and hope it's included before any other
    // 		// transactions in the pool. Next we tweak the priority depending on how much
    // 		// it differs from the current average. (the more it differs the more priority it
    // 		// has).
    // 		.priority(T::UnsignedPriority::get().saturating_add(avg_price as _))
    // 		// This transaction does not require anything else to go before into the pool.
    // 		// In theory we could require `previous_unsigned_at` transaction to go first,
    // 		// but it's not necessary in our case.
    // 		//.and_requires()
    // 		// We set the `provides` tag to be the same as `next_unsigned_at`. This makes
    // 		// sure only one transaction produced after `next_unsigned_at` will ever
    // 		// get to the transaction pool and will end up in the block.
    // 		// We can still have multiple transactions compete for the same "spot",
    // 		// and the one with higher priority will replace other one in the pool.
    // 		.and_provides(next_unsigned_at)
    // 		// The transaction is only valid for next 5 blocks. After that it's
    // 		// going to be revalidated by the pool.
    // 		.longevity(5)
    // 		// It's fine to propagate that transaction to other peers, which means it can be
    // 		// created even by nodes that don't produce blocks.
    // 		// Note that sometimes it's better to keep it for yourself (if you are the block
    // 		// producer), since for instance in some schemes others may copy your solution and
    // 		// claim a reward.
    // 		.propagate(true)
    // 		.build()
    // }
}
