use frame_support::pallet_macros::pallet_section;

/// Define all hooks used in the pallet.
#[pallet_section]
mod hooks {
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            log::info!("Kitties storage migration");
            migration::migrate_to_v1::<T>()
            // Weight::default()
        }

        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            log::info!("Kitties on_initialize at block {:?}", n);
            let _ = Self::trade(n);
            Weight::default()
        }

        fn on_poll(n: BlockNumberFor<T>, _remaining_weight: &mut WeightMeter) {
            log::info!("Kitties on_poll at block {:?}", n);
        }

        fn on_finalize(n: BlockNumberFor<T>) {
            // remove the kitty on sale if no bid and the block number is greater than the until_block.
            // sale the kitty if according to bid price
            log::info!("Kitties on_finalize at block {:?}", n);
        }

        fn on_idle(n: BlockNumberFor<T>, _remaining_weight: Weight) -> Weight {
            log::info!("Kitties on_idle at block {:?}", n);
            Weight::default()
        }

        fn integrity_test() {
            assert!(NextKittyId::<T>::get() == 0);
        }

        fn offchain_worker(n: BlockNumberFor<T>) {
            log::info!("Kitties offchain_worker at block {:?}", n);
            let _ = Self::offchain_worker(n);
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
            log::info!("kitties storage pre_upgrade");
            let kitty_id = NextKittyId::<T>::get();
            Ok(kitty_id.encode())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
            log::info!("kitties storage post_upgrade");
            let kitty_id_before = u32::decode(&mut &state[..]).map_err(|_| "invalid id state")?;
            assert!(
                kitty_id_before == 0 || Kitties::<T>::contains_key(&kitty_id_before),
                "invalid not include state"
            );
            Ok(())
        }

        #[cfg(feature = "try-runtime")]
        fn try_state(_n: BlockNumberFor<T>) -> Result<(), TryRuntimeError> {
            Ok(())
        }
    }
}
