use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, pallet_prelude::*,traits::{Currency,ReservableCurrency,ExistenceRequirement}};

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
        let (owner, bidder, kitty_id, price, until_block) = (1, 2, 1, 500, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));

        // sale kitty & with price 10
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

        run_to_block(2);

        assert_ok!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ));
        assert_eq!(KittiesBid::<Test>::get(kitty_id), Some((bidder, price)));

        run_to_block(until_block);
        assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(bidder));
        System::assert_has_event(Event::<Test>::KittyTransferred { from:owner, to:bidder, kitty_id }.into());
    });
}

#[test]
fn it_failed_for_sale_when_not_enough_balance() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        let (owner, bidder, kitty_id, price, until_block) = (1, 4, 1, 500, 11);

        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));

        // sale kitty & with price 10
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

        run_to_block(2);
      
                let stake_amount= <<Test as Config>::StakeAmount as Get<u128>>::get();

        let _=<Test as Config>::Currency::transfer(&owner,&bidder,<Test as Config>::Currency::minimum_balance() +price+stake_amount,ExistenceRequirement::KeepAlive);
        assert_ok!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ));
        assert_eq!(KittiesBid::<Test>::get(kitty_id), Some((bidder, price)));

        <Test as Config>::Currency::unreserve(&bidder,stake_amount+ 3);


        let _=<Test as Config>::Currency::transfer(&bidder,&owner,stake_amount+ 3, ExistenceRequirement::KeepAlive);

        run_to_block(until_block);
        assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(owner));
    });
}

#[test]
fn create_works() {
    new_test_ext().execute_with(|| {
        let (creator, kitty_id) = (1, 1);
        let origin_reserved_balance = <Test as Config>::Currency::reserved_balance(&creator);
        let origin_free_balance = <Test as Config>::Currency::free_balance(&creator);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        assert_eq!(NextKittyId::<Test>::get(), kitty_id);
        assert!(Kitties::<Test>::get(kitty_id).is_some());
        assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(creator));
        let stake_amount=<<Test as Config>::StakeAmount as Get<u128>>::get();
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&creator),
            origin_reserved_balance + stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&creator),
            origin_free_balance - stake_amount
        );
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
fn create_failed_when_next_kitty_id_overflow() {
    new_test_ext().execute_with(|| {
        let creator = 1;
        NextKittyId::<Test>::put(u32::MAX);
        assert_noop!(PalletKitties::create(RuntimeOrigin::signed(creator)), Error::<Test>::NextKittyIdOverflow);
    });
}

#[test]
fn create_failed_when_not_enough_balance_for_staking() {
    new_test_ext().execute_with(|| {
        let creator = 4;
        assert_noop!(PalletKitties::create(RuntimeOrigin::signed(creator)), Error::<Test>::NotEnoughBalanceForStaking);
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
        let origin_reserved_balance = <Test as Config>::Currency::reserved_balance(&creator);
let origin_free_balance = <Test as Config>::Currency::free_balance(&creator);
        assert_ok!(PalletKitties::breed(
            RuntimeOrigin::signed(creator),
            kitty_id_1,
            kitty_id_2
        ));
        assert_eq!(NextKittyId::<Test>::get(), kitty_id);
  let stake_amount=<<Test as Config>::StakeAmount as Get<u128>>::get();
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&creator),
            origin_reserved_balance + stake_amount
        );
  assert_eq!(
            <Test as Config>::Currency::free_balance(&creator),
            origin_free_balance - stake_amount
        );
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
fn breed_faile_when_same_parent_id() {
    new_test_ext().execute_with(|| {
        let (creator, kitty_id_1) =
            (1, 1);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(
            creator
        )));

        assert_noop!(PalletKitties::breed(
            RuntimeOrigin::signed(creator),
            kitty_id_1,
            kitty_id_1
        ),Error::<Test>::SameParentId);
        
    });
}

#[test]
fn breed_faile_when_kitty1_not_exist() {
    new_test_ext().execute_with(|| {
        let (creator,  kitty_id_1, kitty_id_2) =
            (3, 1, 2);
        assert_noop!(PalletKitties::breed(
            RuntimeOrigin::signed(creator),
            kitty_id_1,
            kitty_id_2
        ),Error::<Test>::KittyNotExist);
        
    });
}
#[test]
fn breed_faile_when_kitty2_not_exist() {
    new_test_ext().execute_with(|| {
        let (creator, kitty_1_account,  kitty_id_1, kitty_id_2) =
            (3, 1, 1, 2);
         assert_ok!(PalletKitties::create(RuntimeOrigin::signed(
            kitty_1_account
        )));
        assert_noop!(PalletKitties::breed(
            RuntimeOrigin::signed(creator),
            kitty_id_1,
            kitty_id_2
        ),Error::<Test>::KittyNotExist);
        
    });
}
#[test]
fn breed_failed_when_next_kitty_id_overflow() {
    new_test_ext().execute_with(|| {
       
let (creator, kitty_1_account, kitty_2_account, kitty_id_1, kitty_id_2) =
            (3, 1, 2, 1, 2);
         assert_ok!(PalletKitties::create(RuntimeOrigin::signed(
            kitty_1_account
        )));
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(
            kitty_2_account
        )));
        NextKittyId::<Test>::put(u32::MAX);
        assert_noop!(PalletKitties::breed(
            RuntimeOrigin::signed(creator),
            kitty_id_1,
            kitty_id_2
        ),Error::<Test>::NextKittyIdOverflow);
    });
}

#[test]
fn breed_failed_when_not_enough_balance_for_staking() {
    new_test_ext().execute_with(|| {
       let (creator, kitty_1_account, kitty_2_account, kitty_id_1, kitty_id_2) =
            (4, 1, 2, 1, 2);
         assert_ok!(PalletKitties::create(RuntimeOrigin::signed(
            kitty_1_account
        )));
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(
            kitty_2_account
        )));
        assert_noop!(PalletKitties::breed(
            RuntimeOrigin::signed(creator),
            kitty_id_1,
            kitty_id_2
        ),Error::<Test>::NotEnoughBalanceForStaking);
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
        let (from, to, kitty_id) = (1, 2, 1);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(from)));
        let stake_amount = <<Test as Config>::StakeAmount as Get<u128>>::get();
        let origin_reserved_balance_of_from = <Test as Config>::Currency::reserved_balance(&from);
        let origin_reserved_balance_of_to = <Test as Config>::Currency::reserved_balance(&to);
let origin_free_balance_of_from = <Test as Config>::Currency::free_balance(&from);
let origin_free_balance_of_to = <Test as Config>::Currency::free_balance(&to);
        assert_ok!(PalletKitties::transfer(
            RuntimeOrigin::signed(from),
            to,
            kitty_id
        ));
        assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(to));

        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&from),
            origin_reserved_balance_of_from - stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&to),
            origin_reserved_balance_of_to + stake_amount
        );
  assert_eq!(
            <Test as Config>::Currency::free_balance(&from),
            origin_free_balance_of_from + stake_amount
        );
  assert_eq!(
            <Test as Config>::Currency::free_balance(&to),
            origin_free_balance_of_to - stake_amount
        );
        System::assert_has_event(Event::<Test>::KittyTransferred { from, to, kitty_id }.into());
    });
}

#[test]
fn transfer_failed_when_kitty_already_on_sale() {
    new_test_ext().execute_with(|| {
        let (from, to, kitty_id,until_block) = (1, 2, 1,11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(from)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(from),
            kitty_id,
            until_block
        ));
        assert_noop!(PalletKitties::transfer(
            RuntimeOrigin::signed(from),
            to,
            kitty_id
        ),Error::<Test>::KittyAlreadyOnSale);
    });
}

#[test]
fn transfer_failed_when_not_owner() {
    new_test_ext().execute_with(|| {
        let (from, to, kitty_id) = (1, 2, 1);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(from)));
        assert_noop!(PalletKitties::transfer(
            RuntimeOrigin::signed(to),
            from,
            kitty_id
        ),Error::<Test>::NotOwner);
    });
}

#[test]
fn transfer_failed_when_not_enough_balance_for_staking() {
    new_test_ext().execute_with(|| {
        let (from, to, kitty_id) = (1, 4, 1);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(from)));
        assert_noop!(PalletKitties::transfer(
            RuntimeOrigin::signed(from),
            to,
            kitty_id
        ),Error::<Test>::NotEnoughBalanceForStaking);
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
        assert!(KittiesBid::<Test>::get(kitty_id).is_none());

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
fn sale_failed_when_not_owner() {
    new_test_ext().execute_with(|| {
        let (owner,other, kitty_id, until_block) = (1,2, 1, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_noop!(PalletKitties::sale(
            RuntimeOrigin::signed(other),
            kitty_id,
            until_block
        ),Error::<Test>::NotOwner);
        
    });
}

#[test]
fn sale_failed_when_kitty_already_on_sale() {
    new_test_ext().execute_with(|| {
        let (owner, kitty_id, until_block) = (1, 1, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));
        assert_noop!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ),Error::<Test>::KittyAlreadyOnSale);
        
    });
}

#[test]
fn sale_failed_when_block_span_too_small() {
    new_test_ext().execute_with(|| {
        let (owner, kitty_id, until_block) = (1, 1, 10);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_noop!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ),Error::<Test>::BlockSpanTooSmall);
        
    });
}

#[test]
fn sale_failed_when_too_many_bid_on_one_block() {
    new_test_ext().execute_with(|| {
        let (owner, kitty_id, until_block) = (1, 11, 11);
        for id in 1..=10{
            assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
            assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            id,
            until_block
            ));
        }
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_noop!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ),Error::<Test>::TooManyBidOnOneBlock);
        
    });
}

#[test]
fn bid_works() {
    new_test_ext().execute_with(|| {
        let (owner, bidder, kitty_id, price, until_block) = (1, 2, 1, 500, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));
        let origin_reserved_balance = <Test as Config>::Currency::reserved_balance(&bidder);
        let origin_free_balance = <Test as Config>::Currency::free_balance(&bidder);
        assert_ok!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ));
        assert_eq!(KittiesBid::<Test>::get(kitty_id), Some((bidder, price)));
        let stake_amount= <<Test as Config>::StakeAmount as Get<u128>>::get();
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&bidder),
            origin_reserved_balance + price + stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&bidder),
            origin_free_balance -price- stake_amount
        );
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

#[test]
fn bid_failed_when_bid_for_self() {
    new_test_ext().execute_with(|| {
        let (owner,  kitty_id, price, until_block) = (1, 1, 500, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));

        assert_noop!(PalletKitties::bid(
            RuntimeOrigin::signed(owner),
            kitty_id,
            price
        ),Error::<Test>::BidForSelf);
       
    });
}


#[test]
fn bid_failed_when_kitty_not_on_sale() {
    new_test_ext().execute_with(|| {
        let (owner, bidder, kitty_id, price) = (1, 2, 1, 500);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_noop!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ),Error::<Test>::KittyNotOnSale);
       
    });
}


#[test]
fn bid_failed_when_kitty_bid_less_than_or_minimum_bid_amount() {
    new_test_ext().execute_with(|| {
        let (owner, bidder, kitty_id, price, until_block) = (1, 4, 1, 2, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));

        assert_noop!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ),Error::<Test>::KittyBidLessThanOrMinimumBidAmount);
       
    });
}


#[test]
fn bid_failed_when_kitty_bid_less_than_the_sum_of_last_price_and_minimum_bid_increment() {
    new_test_ext().execute_with(|| {
        let (owner, bidder,bidder2, kitty_id, price, until_block) = (1,2,4, 1, 500, 11);
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
        assert_noop!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder2),
            kitty_id,
            price
        ),Error::<Test>::KittyBidLessThanTheSumOfLastPriceAndMinimumBidIncrement);
       
    });
}

#[test]
fn bid_failed_when_not_enough_balance_for_bid_and_staking() {
    new_test_ext().execute_with(|| {
        let (owner, bidder, kitty_id, price, until_block) = (1, 4, 1, 500, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));

        assert_noop!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ),Error::<Test>::NotEnoughBalanceForBidAndStaking);
       
    });
}