use frame_support::pallet_macros::pallet_section;

/// Define all extrinsics for the pallet.
#[pallet_section]
mod dispatches {
    /// Dispatchable functions allow users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        pub fn create(origin: OriginFor<T>, something: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let _value = Self::random_value(&who);

            // Something::<T>::put(something);

            // Self::deposit_event(Event::SomethingStored { something, who });

            Ok(())
        }

        pub fn breed(origin: OriginFor<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            Ok(())
        }

        pub fn transfer(origin: OriginFor<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            Ok(())
        }

        pub fn sale(
            origin: OriginFor<T>,
            kitty_id: u32,
            until_block: BlockNumberFor<T>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            Ok(())
        }

        pub fn bid(origin: OriginFor<T>, kitty_id: u32, price: u64) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            Ok(())
        }
    }
}
