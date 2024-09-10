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
use alloc::vec::Vec;
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


use sp_core::{crypto::KeyTypeId};

// ...

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");

// ...

pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify, MultiSignature, MultiSigner
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	// implemented for runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
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
    use frame_system::pallet_prelude::*;
use frame_system::offchain::SendSignedTransaction;

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
    pub trait Config: frame_system::offchain::CreateSignedTransaction<Call<Self>> + frame_system::Config {
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
let signer = frame_system::offchain::Signer::<T, T::AuthorityId>::all_accounts();
	// Using `send_signed_transaction` associated type we create and submit a transaction
	// representing the call we've just created.
	// `send_signed_transaction()` return type is `Option<(Account<T>, Result<(), ()>)>`. It is:
	//	 - `None`: no account is available for sending transaction
	//	 - `Some((account, Ok(())))`: transaction is successfully sent
	//	 - `Some((account, Err(())))`: error occurred when sending the transaction

	let results = signer.send_signed_transaction(|_account| {
		 Call::submit_data { payload: Vec::from([0]) }
	});
	for (acc, res) in &results {
		match res {
			Ok(()) => log::info!("[{:?}]: submit transaction success.", acc.id),
			Err(e) => log::error!("[{:?}]: submit transaction failure. Reason: {:?}", acc.id, e),
		}
	}

	// Ok(())


// let value: u64 = 10;
// 		// This is your call to on-chain extrinsic together with any necessary parameters.
// 		let call = RuntimeCall::unsigned_extrinsic1 { key: value };

// 		// `submit_unsigned_transaction` returns a type of `Result<(), ()>`
// 		//	 ref: https://paritytech.github.io/substrate/master/frame_system/offchain/struct.SubmitTransaction.html
// 		SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
// 			.map_err(|_| {
// 			log::error!("Failed in offchain_unsigned_tx");
// 		});



// let value: u64 = 10;

// 		// Retrieve the signer to sign the payload
// 		let signer = Signer::<T, T::AuthorityId>::any_account();

// 		// `send_unsigned_transaction` is returning a type of `Option<(Account<T>, Result<(), ()>)>`.
// 		//	 The returned result means:
// 		//	 - `None`: no account is available for sending transaction
// 		//	 - `Some((account, Ok(())))`: transaction is successfully sent
// 		//	 - `Some((account, Err(())))`: error occurred when sending the transaction
// 		if let Some((_, res)) = signer.send_unsigned_transaction(
// 			// this line is to prepare and return payload
// 			|acct| Payload { number, public: acct.public.clone() },
// 			|payload, signature| RuntimeCall::some_extrinsics { payload, signature },
// 		) {
// 			match res {
// 				Ok(()) => log::info!("unsigned tx with signed payload successfully sent.");
// 				Err(()) => log::error!("sending unsigned tx with signed payload failed.");
// 			};
// 		} else {
// 			// The case of `None`: no account is available for sending
// 			log::error!("No local account available");
// 		}
	}

}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
pub struct Payload<Public> {
	number: u64,
	public: Public,
}

impl<T: frame_system::offchain::SigningTypes> frame_system::offchain::SignedPayload<T> for Payload<T::Public> {
	fn public(&self) -> T::Public {
	self.public.clone()
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
            let UNSIGNED_TXS_PRIORITY=3;
			let valid_tx = |provide| ValidTransaction::with_tag_prefix("my-pallet")
		.priority(UNSIGNED_TXS_PRIORITY) // please define `UNSIGNED_TXS_PRIORITY` before this line
		.and_provides([&provide])
		.longevity(3)
		.propagate(true)
		.build();
match call {
			Call::unsigned_extrinsic_with_signed_payload {
			ref payload,
			ref signature
			} => {
			if !frame_system::offchain::SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
				return InvalidTransaction::BadProof.into();
			}
			valid_tx(b"unsigned_extrinsic_with_signed_payload".to_vec())
			},
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
		pub fn unsigned_extrinsic_with_signed_payload(origin: OriginFor<T>, payload: Payload<T::Public>, _signature: T::Signature,) -> DispatchResult {
			ensure_none(origin)?;

            log::info!("OCW ==> in call unsigned_extrinsic_with_signed_payload: {:?}", payload.number);
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
    }
}
