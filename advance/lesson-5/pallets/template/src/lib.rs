//! # Template Pallet
//!
//! A pallet with minimal functionality to help developers understand the essential components of
//! writing a FRAME pallet. It is typically used in beginner tutorials or in Polkadot SDK template
//! as a starting point for creating a new pallet and **not meant to be used in production**.
//!
//! ## Overview
//!
//! This template pallet contains basic examples of:
//! - declaring a storage item that stores a single block-number
//! - declaring and using events
//! - declaring and using errors
//! - a dispatchable function that allows a user to set a new value to storage and emits an event
//!   upon success
//! - another dispatchable function that causes a custom error to be thrown
//!
//! Each pallet section is annotated with an attribute using the `#[pallet::...]` procedural macro.
//! This macro generates the necessary code for a pallet to be aggregated into a FRAME runtime.
//!
//! To get started with pallet development, consider using this tutorial:
//!
//! <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html>
//!
//! And reading the main documentation of the `frame` crate:
//!
//! <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/polkadot_sdk/frame_runtime/index.html>
//!
//! And looking at the frame [`kitchen-sink`](https://paritytech.github.io/polkadot-sdk/master/pallet_example_kitchensink/index.html)
//! pallet, a showcase of all pallet macros.
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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{vec, vec::Vec};
use cumulus_pallet_xcm::{ensure_sibling_para, Origin as CumulusOrigin};
use cumulus_primitives_core::ParaId;
use frame_support::{parameter_types, BoundedVec};
use frame_system::Config as SystemConfig;
pub use pallet::*;
use sp_runtime::traits::Saturating;
use xcm::latest::prelude::*;
parameter_types! {
    const MaxParachains: u32 = 100;
    const MaxPayloadSize: u32 = 1024;
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/polkadot_sdk/frame_runtime/index.html>
// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html>
//
// To see a full list of `pallet` macros and their use cases, see:
// <https://paritytech.github.io/polkadot-sdk/master/pallet_example_kitchensink/index.html>
// <https://paritytech.github.io/polkadot-sdk/master/frame_support/pallet_macros/index.html>
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, DefaultNoBound};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{CheckedAdd, One};

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        /// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type RuntimeOrigin: From<<Self as SystemConfig>::RuntimeOrigin>
            + Into<Result<CumulusOrigin, <Self as Config>::RuntimeOrigin>>;

        /// The overarching call type; we assume sibling chains use the same type.
        type RuntimeCall: From<Call<Self>> + Encode;

        type XcmSender: SendXcm;
        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// A struct to store a single block-number. Has all the right derives to store it in storage.
    /// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_storage_derives/index.html>
    #[derive(
        Encode, Decode, MaxEncodedLen, TypeInfo, CloneNoBound, PartialEqNoBound, DefaultNoBound,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct CompositeStruct<T: Config> {
        /// A block number.
        pub(crate) block_number: BlockNumberFor<T>,
    }

    /// The pallet's storage items.
    /// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#storage>
    /// <https://paritytech.github.io/polkadot-sdk/master/frame_support/pallet_macros/attr.storage.html>
    #[pallet::storage]
    pub type Something<T: Config> = StorageValue<_, CompositeStruct<T>>;

    /// The target parachains to ping.
    #[pallet::storage]
    pub(super) type Targets<T: Config> = StorageValue<
        _,
        BoundedVec<(ParaId, BoundedVec<u8, MaxPayloadSize>), MaxParachains>,
        ValueQuery,
    >;

    /// The total number of pings sent.
    #[pallet::storage]
    pub(super) type PingCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// The sent pings.
    #[pallet::storage]
    pub(super) type Pings<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, BlockNumberFor<T>, OptionQuery>;

    /// Pallets use events to inform users when important changes are made.
    /// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// We usually use passive tense for events.
        SomethingStored {
            block_number: BlockNumberFor<T>,
            who: T::AccountId,
        },
        PingSent(ParaId, u32, Vec<u8>, XcmHash, Assets),
        Pinged(ParaId, u32, Vec<u8>),
        PongSent(ParaId, u32, Vec<u8>, XcmHash, Assets),
        Ponged(ParaId, u32, Vec<u8>, BlockNumberFor<T>),
        ErrorSendingPing(SendError, ParaId, u32, Vec<u8>),
        ErrorSendingPong(SendError, ParaId, u32, Vec<u8>),
        UnknownPong(ParaId, u32, Vec<u8>),
    }

    /// Errors inform users that something went wrong.
    /// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        /// Too many parachains have been added as a target.
        TooManyTargets,
        /// The payload provided is too large, limit is 1024 bytes.
        PayloadTooLarge,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(n: BlockNumberFor<T>) {
            for (para, payload) in Targets::<T>::get().into_iter() {
                let seq = PingCount::<T>::mutate(|seq| {
                    *seq += 1;
                    *seq
                });
                match send_xcm::<T::XcmSender>(
                    (Parent, Junction::Parachain(para.into())).into(),
                    Xcm(vec![Transact {
                        origin_kind: OriginKind::Native,
                        require_weight_at_most: Weight::from_parts(1_000, 1_000),
                        call: <T as Config>::RuntimeCall::from(Call::<T>::ping {
                            seq,
                            payload: payload.clone().to_vec(),
                        })
                        .encode()
                        .into(),
                    }]),
                ) {
                    Ok((hash, cost)) => {
                        Pings::<T>::insert(seq, n);
                        Self::deposit_event(Event::PingSent(
                            para,
                            seq,
                            payload.to_vec(),
                            hash,
                            cost,
                        ));
                    }
                    Err(e) => {
                        Self::deposit_event(Event::ErrorSendingPing(
                            e,
                            para,
                            seq,
                            payload.to_vec(),
                        ));
                    }
                }
            }
        }
    }

    /// Dispatchable functions allows users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    /// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#dispatchables>
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn start(origin: OriginFor<T>, para: ParaId, payload: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            let payload = BoundedVec::<u8, MaxPayloadSize>::try_from(payload)
                .map_err(|_| Error::<T>::PayloadTooLarge)?;
            Targets::<T>::try_mutate(|t| {
                t.try_push((para, payload))
                    .map_err(|_| Error::<T>::TooManyTargets)
            })?;
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        pub fn start_many(
            origin: OriginFor<T>,
            para: ParaId,
            count: u32,
            payload: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let bounded_payload = BoundedVec::<u8, MaxPayloadSize>::try_from(payload)
                .map_err(|_| Error::<T>::PayloadTooLarge)?;
            for _ in 0..count {
                Targets::<T>::try_mutate(|t| {
                    t.try_push((para, bounded_payload.clone()))
                        .map_err(|_| Error::<T>::TooManyTargets)
                })?;
            }
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight({0})]
        pub fn stop(origin: OriginFor<T>, para: ParaId) -> DispatchResult {
            ensure_root(origin)?;
            Targets::<T>::mutate(|t| {
                if let Some(p) = t.iter().position(|(p, _)| p == &para) {
                    t.swap_remove(p);
                }
            });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight({0})]
        pub fn stop_all(origin: OriginFor<T>, maybe_para: Option<ParaId>) -> DispatchResult {
            ensure_root(origin)?;
            if let Some(para) = maybe_para {
                Targets::<T>::mutate(|t| t.retain(|&(x, _)| x != para));
            } else {
                Targets::<T>::kill();
            }
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight({0})]
        pub fn ping(origin: OriginFor<T>, seq: u32, payload: Vec<u8>) -> DispatchResult {
            // Only accept pings from other chains.
            let para = ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin))?;

            Self::deposit_event(Event::Pinged(para, seq, payload.clone()));
            match send_xcm::<T::XcmSender>(
                (Parent, Junction::Parachain(para.into())).into(),
                Xcm(vec![Transact {
                    origin_kind: OriginKind::Native,
                    require_weight_at_most: Weight::from_parts(1_000, 1_000),
                    call: <T as Config>::RuntimeCall::from(Call::<T>::pong {
                        seq,
                        payload: payload.clone(),
                    })
                    .encode()
                    .into(),
                }]),
            ) {
                Ok((hash, cost)) => {
                    Self::deposit_event(Event::PongSent(para, seq, payload, hash, cost))
                }
                Err(e) => Self::deposit_event(Event::ErrorSendingPong(e, para, seq, payload)),
            }
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight({0})]
        pub fn pong(origin: OriginFor<T>, seq: u32, payload: Vec<u8>) -> DispatchResult {
            // Only accept pings from other chains.
            let para = ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin))?;

            if let Some(sent_at) = Pings::<T>::take(seq) {
                Self::deposit_event(Event::Ponged(
                    para,
                    seq,
                    payload,
                    frame_system::Pallet::<T>::block_number().saturating_sub(sent_at),
                ));
            } else {
                // Pong received for a ping we apparently didn't send?!
                Self::deposit_event(Event::UnknownPong(para, seq, payload));
            }
            Ok(())
        }
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn do_something(origin: OriginFor<T>, bn: u32) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_origin/index.html>
            let who = ensure_signed(origin)?;

            // Convert the u32 into a block number. This is possible because the set of trait bounds
            // defined in [`frame_system::Config::BlockNumber`].
            let block_number: BlockNumberFor<T> = bn.into();

            // Update storage.
            <Something<T>>::put(CompositeStruct { block_number });

            // Emit an event.
            Self::deposit_event(Event::SomethingStored { block_number, who });

            // Return a successful [`DispatchResultWithPostInfo`] or [`DispatchResult`].
            Ok(().into())
        }

        /// An example dispatchable that may throw a custom error.
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
        pub fn cause_error(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;

            // Read a value from storage.
            match <Something<T>>::get() {
                // Return an error if the value has not been set.
                None => Err(Error::<T>::NoneValue)?,
                Some(mut old) => {
                    // Increment the value read from storage; will error in the event of overflow.
                    old.block_number = old
                        .block_number
                        .checked_add(&One::one())
                        // ^^ equivalent is to:
                        // .checked_add(&1u32.into())
                        // both of which build a `One` instance for the type `BlockNumber`.
                        .ok_or(Error::<T>::StorageOverflow)?;
                    // Update the value in storage with the incremented result.
                    <Something<T>>::put(old);
                    // Explore how you can rewrite this using
                    // [`frame_support::storage::StorageValue::mutate`].
                    Ok(().into())
                }
            }
        }
    }
}
