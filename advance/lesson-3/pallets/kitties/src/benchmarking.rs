//! Benchmarking setup for pallet-kitties
#![cfg(feature = "runtime-benchmarks")]
use super::*;
 use sp_std::vec;
#[allow(unused)]
use crate::Pallet as PalletKitties;
use frame_benchmarking::v2::*;
use frame_system::{pallet_prelude::BlockNumberFor,RawOrigin};
use frame_support::{assert_noop, assert_ok, pallet_prelude::*};
use frame_support::traits::{Currency, EnsureOrigin, Get, ReservableCurrency};

fn create_funded_user<T: Config>(
	string: &'static str,
	n: u32,
	balance_factor: u32,
) -> T::AccountId {
	let user = account(string, n, 0);
	let balance = T::Currency::minimum_balance() * balance_factor.into();
	let _ = T::Currency::make_free_balance_be(&user, balance);
	user
}
fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}
#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create() {
        let (creator, kitty_id):_ = (whitelisted_caller::<T::AccountId>(), 1);
        T::Currency::make_free_balance_be(&creator, T::Currency::minimum_balance() * 3000u32.into());
        #[extrinsic_call]
        create(RawOrigin::Signed(creator.clone()));

        assert_eq!(NextKittyId::<T>::get(), kitty_id);
        assert!(Kitties::<T>::get(kitty_id).is_some());
        assert_eq!(KittyOwner::<T>::get(kitty_id).as_ref(), Some(&creator));
        assert_has_event::<T>(
            Event::<T>::KittyCreated {
                creator,
                kitty_id,
                data: Kitties::<T>::get(kitty_id).unwrap().0.clone(),
            }
            .into(),
        );
    }

    #[benchmark]
    fn breed() {
        let (creator, kitty_1_account, kitty_2_account, kitty_id_1, kitty_id_2, kitty_id) = (
            whitelisted_caller::<T::AccountId>(),
            create_funded_user::<T>("kitty1", 0, 1000),
            create_funded_user::<T>("kitty2", 0, 1000),
            1,
            2,
            3,
        );
        T::Currency::make_free_balance_be(&creator, T::Currency::minimum_balance() * 3000u32.into());

        assert_ok!(Pallet::<T>::create(RawOrigin::Signed(kitty_1_account).into()));
        assert_ok!(Pallet::<T>::create(RawOrigin::Signed(kitty_2_account).into()));
        #[extrinsic_call]
        breed(RawOrigin::Signed(creator.clone()), kitty_id_1, kitty_id_2);

        assert_eq!(NextKittyId::<T>::get(), kitty_id);

        assert_has_event::<T>(
            Event::<T>::KittyCreated {
                creator,
                kitty_id,
                data: Kitties::<T>::get(kitty_id).unwrap().0.clone(),
            }
            .into(),
        );
    }

    #[benchmark]
    fn transfer() {
        let (from, to, kitty_id):_ = (whitelisted_caller::<T::AccountId>(), create_funded_user::<T>("to", 0, 1000), 1);
        T::Currency::make_free_balance_be(&from, T::Currency::minimum_balance() * 3000u32.into());

        assert_ok!(Pallet::<T>::create(RawOrigin::Signed(from.clone()).into()));
        #[extrinsic_call]
        transfer(RawOrigin::Signed(from.clone()), to.clone(), kitty_id);

        assert_eq!(KittyOwner::<T>::get(kitty_id).as_ref(), Some(&to));

        assert_has_event::<T>(Event::<T>::KittyTransferred { from, to, kitty_id }.into());
    }

    #[benchmark]
    fn sale() {
        let new_block: BlockNumberFor<T> = 11u32.into();
        let (owner, kitty_id, until_block):_ = (whitelisted_caller::<T::AccountId>(), 1,new_block);
        T::Currency::make_free_balance_be(&owner, T::Currency::minimum_balance() * 3000u32.into());

        assert_ok!(Pallet::<T>::create(RawOrigin::Signed(owner.clone()).into()));
        #[extrinsic_call]
        sale(RawOrigin::Signed(owner.clone()), kitty_id, until_block);

        assert_eq!(
            KittiesOnSale::<T>::get(&until_block),
            BoundedVec::<u32, <T as Config>::MaxKittiesBidPerBlock>::try_from(vec![kitty_id])
                .unwrap()
        );

        assert_has_event::<T>(
            Event::<T>::KittyOnSale {
                owner,
                kitty_id,
                until_block,
            }
            .into(),
        );
    }

    #[benchmark]
    fn bid() {
 let new_block: BlockNumberFor<T> = 11u32.into();
        let balance_price:BalanceOf<T>=20u32.into();
        let (owner, bidder, kitty_id, price, until_block):_ =
            (whitelisted_caller::<T::AccountId>(), create_funded_user::<T>("bidder", 0, 1000), 1, balance_price, new_block);
        T::Currency::make_free_balance_be(&owner, T::Currency::minimum_balance() * 3000u32.into());

        assert_ok!(Pallet::<T>::create(RawOrigin::Signed(owner.clone()).into()));
        assert_ok!(Pallet::<T>::sale(
            RawOrigin::Signed(owner.clone()).into(),
            kitty_id,
            until_block
        ));

        #[extrinsic_call]
        bid(RawOrigin::Signed(bidder.clone()), kitty_id, price);

        assert_eq!(KittiesBid::<T>::get(kitty_id), Some((bidder.clone(), price)));

        assert_has_event::<T>(
            Event::<T>::KittyBid {
                bidder,
                kitty_id,
                price,
            }
            .into(),
        );
    }
   

    impl_benchmark_test_suite!(PalletKitties, crate::mock::new_test_ext(), crate::mock::Test);
}
