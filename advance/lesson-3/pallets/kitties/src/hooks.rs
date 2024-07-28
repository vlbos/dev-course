use frame_support::pallet_macros::pallet_section;

/// Define all hooks used in the pallet.
#[pallet_section]
mod hooks {
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            log::info!("Kitties on_initialize at block {:?}", n);
            Weight::default()
        }
        fn on_finalize(n: BlockNumberFor<T>) {
            log::info!("Kitties on_finalize at block {:?}", n);
        }
        fn on_runtime_upgrade() -> Weight {
            Weight::default()
        }
        fn integrity_test() {}
    }
}
