use frame_support::pallet_macros::pallet_section;

#[pallet_section]
mod hooks {
    // use super::*;
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            log::info!("on_initialize at block {}", n);
            0.into()
        }
        fn on_finalize(n: BlockNumberFor<T>) {}
        fn on_runtime_upgrade() -> Weight {
            0.into()
        }
        fn integrity_test() {}
    }
}
