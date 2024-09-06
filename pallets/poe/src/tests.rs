use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_err, assert_noop, assert_ok, BoundedVec};
const ALICE: u64 = 1;
const BOB: u64 = 2;
const CAROL: u64 = 3;

#[test]
fn create_claim_works_with_deposited_event() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let claim = BoundedVec::try_from(vec![1, 2, 3]).unwrap();
		let sender = ALICE;
		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(sender), claim.clone()));

		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((sender, frame_system::Pallet::<Test>::block_number()))
		);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::ClaimCreated(sender, claim.clone()).into());
	})
}

#[test]
fn create_claim_works_with_empty_vec_claim() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let claim = BoundedVec::try_from(vec![]).unwrap();
		let sender = ALICE;
		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(sender), claim.clone()));

		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((sender, frame_system::Pallet::<Test>::block_number()))
		);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::ClaimCreated(sender, claim.clone()).into());
	})
}

#[test]
fn create_claim_works_with_max_length_claim() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
        let mx_len = <<Test as Config>::MaxClaimLength as Get<u32>>::get() as usize;
		let claim = BoundedVec::try_from(vec![0xff;mx_len]).unwrap();
		let sender = ALICE;
		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(sender), claim.clone()));

		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((sender, frame_system::Pallet::<Test>::block_number()))
		);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::ClaimCreated(sender, claim.clone()).into());
	})
}

#[test]
fn create_claim_failed_when_claim_already_exist_with_storage() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![]).unwrap();
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(BOB), claim.clone());

		//The storage state before executing
		assert!(Proofs::<Test>::get(&claim).is_some());
		assert_noop!(
			PoeModule::create_claim(RuntimeOrigin::signed(ALICE), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
		//The storage state after executing
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((BOB, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

#[test]
fn create_claim_failed_when_the_length_of_the_claim_is_larger_than_the_max_claim_length() {
	new_test_ext().execute_with(|| {
		let mx_len = <<Test as Config>::MaxClaimLength as Get<u32>>::get() as usize;

		let claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(vec![0; mx_len + 1]);
		assert_err!(claim, vec![0; mx_len + 1]);
	})
}

#[test]
fn revoke_claim_works_with_deposited_event() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		let claim = BoundedVec::try_from(vec![]).unwrap();
		let sender = ALICE;
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(sender), claim.clone());
		// The  stoarge  state before the claim is revoked
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((sender, frame_system::Pallet::<Test>::block_number()))
		);
		assert_ok!(PoeModule::revoke_claim(RuntimeOrigin::signed(sender), claim.clone()));
		// The  stoarge  state after the claim is revoked
		assert_eq!(Proofs::<Test>::get(&claim), None);

		// ClaimRevoked Event is raised
		assert!(System::events().iter().any(
			|e| e.event == RuntimeEvent::PoeModule(Event::ClaimRevoked(sender, claim.clone()))
		));
	})
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist_with_storage() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![]).unwrap();
		// The  stoarge  state before executed failed
		assert!(Proofs::<Test>::get(&claim).is_none());
		assert_noop!(
			PoeModule::revoke_claim(RuntimeOrigin::signed(ALICE), claim.clone()),
			Error::<Test>::ClaimNotExist
		);
		// The  stoarge  state before executed failed
		assert!(Proofs::<Test>::get(&claim).is_none());
	})
}

#[test]
fn revoke_claim_failed_with_wrong_owner_with_storage() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![]).unwrap();
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(ALICE), claim.clone());

		// The  stoarge state before the claim is revoked
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((ALICE, frame_system::Pallet::<Test>::block_number()))
		);
		assert_noop!(
			PoeModule::revoke_claim(RuntimeOrigin::signed(BOB), claim.clone()),
			Error::<Test>::NotClaimOwner
		);
		// The  stoarge state after the claim is revoked
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((ALICE, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

#[test]
fn transfer_claim_works_with_storage_and_claim_of_the_max_length() {
	new_test_ext().execute_with(|| {
		let mx_len = <<Test as Config>::MaxClaimLength as Get<u32>>::get() as usize;
		let Ok(claim) = BoundedVec::try_from(vec![0xff; mx_len]) else {
			panic!(
				"The length of the claim must be less than or equal to `MaxClaimLength({})`",
				mx_len
			);
		};
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(ALICE), claim.clone());
		// The  stoarge state Before the claim is transferred
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((ALICE, frame_system::Pallet::<Test>::block_number()))
		);
		assert_ok!(PoeModule::transfer_claim(RuntimeOrigin::signed(ALICE), claim.clone(), BOB));

		// The stoarge state After the claim is transferred
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((BOB, frame_system::Pallet::<Test>::block_number()))
		);
	})
}
#[test]
fn transfer_claim_failed_when_claim_is_not_exist_with_storage() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![]).unwrap();
		// The  stoarge  state before executed failed
		assert!(Proofs::<Test>::get(&claim).is_none());
		assert_noop!(
			PoeModule::transfer_claim(RuntimeOrigin::signed(ALICE), claim.clone(), BOB),
			Error::<Test>::ClaimNotExist
		);
		// The  stoarge  state after executed failed
		assert!(Proofs::<Test>::get(&claim).is_none());
	})
}

#[test]
fn transfer_claim_failed_with_wrong_owner_with_storage() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![]).unwrap();
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(ALICE), claim.clone());

		// The  stoarge state before the executed failed
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((ALICE, frame_system::Pallet::<Test>::block_number()))
		);
		assert_noop!(
			PoeModule::transfer_claim(RuntimeOrigin::signed(BOB), claim.clone(), CAROL),
			Error::<Test>::NotClaimOwner
		);
		// The  stoarge state after the executed failed
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((ALICE, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

// The Original Part
#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(ALICE), claim.clone()));

		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
		assert_eq!(<<Test as Config>::MaxClaimLength as Get<u32>>::get(), 10);
	})
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(ALICE), claim.clone());

		assert_noop!(
			PoeModule::create_claim(RuntimeOrigin::signed(ALICE), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	})
}

#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(ALICE), claim.clone());

		assert_ok!(PoeModule::revoke_claim(RuntimeOrigin::signed(ALICE), claim.clone()));
	})
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();

		assert_noop!(
			PoeModule::revoke_claim(RuntimeOrigin::signed(ALICE), claim.clone()),
			Error::<Test>::ClaimNotExist
		);
	})
}

#[test]
fn revoke_claim_failed_with_wrong_owner() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(ALICE), claim.clone());

		assert_noop!(
			PoeModule::revoke_claim(RuntimeOrigin::signed(BOB), claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	})
}

#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(ALICE), claim.clone());

		assert_ok!(PoeModule::transfer_claim(RuntimeOrigin::signed(ALICE), claim.clone(), BOB));

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((BOB, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

#[test]
fn transfer_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();

		assert_noop!(
			PoeModule::transfer_claim(RuntimeOrigin::signed(ALICE), claim.clone(), BOB),
			Error::<Test>::ClaimNotExist
		);
	})
}

#[test]
fn transfer_claim_failed_with_wrong_owner() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
		let _ = PoeModule::create_claim(RuntimeOrigin::signed(ALICE), claim.clone());

		assert_noop!(
			PoeModule::transfer_claim(RuntimeOrigin::signed(BOB), claim.clone(), CAROL),
			Error::<Test>::NotClaimOwner
		);
	})
}
