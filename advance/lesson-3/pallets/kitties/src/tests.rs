use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, pallet_prelude::*};

use super::*;
#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        run_to_block(2);
    });
}
#[test]
fn it_works_for_sale() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        let sale_account: u64 = 1;
        let bid_account: u64 = 2;

        // assert_ok!(Kittles::create(<<Test as Config>::RuntimeOrigin>::signed(
        //     sale_account
        // )));
        // // sale kitty & with price 10
        // assert_ok!(Kitties::sale(
        //     <<Test as config>::Runtimeorigin>::signed(sale_account),
        //     0,
        //     10
        // ));
        // assert_eq!(crate::KittyOnSale::<Test>::get(0), Some(10));

        // run_to_block(2);

        // assert_ok!(Kitties::bid(<Test as Config>::RuntimcOrigin>>::signed(bid_account),0,100));

        // run_to_block(10);
        // assert_eq!(crate::KittyOwner::<Test>::get(0), Some(bid_account));
    });
}

#[test]
fn create_works() {
    new_test_ext().execute_with(|| {
        let (creator, kitty_id) = (1, 1);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        assert_eq!(NextKittyId::<Test>::get(), kitty_id);
        assert!(Kitties::<Test>::get(kitty_id).is_some());
        assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(creator));
        System::assert_has_event(
            Event::<Test>::KittyCreated {
                creator,
                kitty_id,
                data: Kitties::<Test>::get(kitty_id).unwrap().0.clone(),
            }
            .into(),
        );
    });
}

#[test]
fn breed_works() {
    new_test_ext().execute_with(|| {
        let (creator, kitty_1_account, kitty_2_account, kitty_id_1, kitty_id_2, kitty_id) =
            (3, 1, 2, 1, 2, 3);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(
            kitty_1_account
        )));
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(
            kitty_2_account
        )));
        assert_ok!(PalletKitties::breed(
            RuntimeOrigin::signed(creator),
            kitty_id_1,
            kitty_id_2
        ));
        assert_eq!(NextKittyId::<Test>::get(), kitty_id);

        System::assert_has_event(
            Event::<Test>::KittyCreated {
                creator,
                kitty_id,
                data: Kitties::<Test>::get(kitty_id).unwrap().0.clone(),
            }
            .into(),
        );
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
        let (from, to, kitty_id) = (1, 2, 1);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(from)));
        assert_ok!(PalletKitties::transfer(
            RuntimeOrigin::signed(from),
            to,
            kitty_id
        ));
        assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(to));

        System::assert_has_event(Event::<Test>::KittyTransferred { from, to, kitty_id }.into());
    });
}

#[test]
fn sale_works() {
    new_test_ext().execute_with(|| {
        let (owner, kitty_id, until_block) = (1, 1, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));
        assert_eq!(
            KittiesOnSale::<Test>::get(&until_block),
            BoundedVec::<u32, <Test as Config>::MaxKittiesBidPerBlock>::try_from(vec![kitty_id])
                .unwrap()
        );

        System::assert_has_event(
            Event::<Test>::KittyOnSale {
                owner,
                kitty_id,
                until_block,
            }
            .into(),
        );
    });
}

#[test]
fn bid_works() {
    new_test_ext().execute_with(|| {
        let (owner, bidder, kitty_id, price, until_block) = (1, 2, 1, 20, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));
        assert_ok!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ));
        assert_eq!(KittiesBid::<Test>::get(kitty_id), Some((bidder, price)));

        System::assert_has_event(
            Event::<Test>::KittyBid {
                bidder,
                kitty_id,
                price,
            }
            .into(),
        );
    });
}
