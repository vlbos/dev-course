
use crate::{Config, Kitties, Kitty, Pallet};
use frame_support::{pallet_prelude::*, storage_alias};
use sp_std::prelude::*;
// use storage::IterableStorageMap;
mod v0 {
    use super::*;
    // only contains V0 storage format
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct OldKitty(pub [u8; 16]);
    #[storage_alias]
    pub type Kitties<T: Config, OldKitty> = StorageMap<Pallet<T>, Blake2_128Concat, u32, OldKitty>;
}

// If migrate fron vo pub fn migrate_to_v1<t: Config>)
pub fn migrate_to_v1<T: Config>() -> Weight {
    let on_chain: StorageVersion = Pallet::<T>::on_chain_storage_version();
    if on_chain == 0 {
        log::info!("current version is 0, will upgrade to v1");
        log::info!(
            "current version is 0, will upgrade to v1,Old Kitties len:{:?}",
            v0::Kitties::<T, v0::OldKitty>::iter().count()
        );
        // for (key, value) in v0::Kitties::<T,v0::OldKitty>::drain() {
        //     log::info!("current version is 0, will upgrade to v1,new Kitties id:{:?}",key);
        //     // let new_kitty: Kitty<T> = Kitty {
        //     //     dna: value.0,
        //     //     price: None,
        //     // };
        //     // Kitties::<T>::insert(key, new_kitty);
        // }
        Kitties::<T>::translate::<v0::OldKitty, _>(|key: u32, value: v0::OldKitty| {
            log::info!(
                " translate current version is 0, will upgrade to v1,Old Kitties id:{:?}",
                key
            );
            Some(Kitty {
                dna: value.0,
                price: None,
            })
        });
        StorageVersion::new(1).put::<Pallet<T>>();
        let count = Kitties::<T>::iter().count() as u64 + 1;
        log::info!(
            "current version is 0, will upgrade to v1,Kitties len:{:?}",
            count
        );
        return T::DbWeight::get().reads_writes(count, count);
    }
    Weight::default()
}
