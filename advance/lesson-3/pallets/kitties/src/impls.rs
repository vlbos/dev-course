use frame_support::pallet_macros::pallet_section;

/// Define the implementation of the pallet, like helper functions.
#[pallet_section]
mod impls {
    impl<T: Config> Pallet<T> {
        // get a random 256.
        fn random_value() -> [u8; 16] {
            // let payload = (
            //     T::Randomness::random_seed(),
            //     &sender,
            //     <frame_system::Pallet<T>>::extrinsic_index(),
            // );

            // payload.using_encoded(blake2_128)
            [0_u8; 16]
        }
    }
}
