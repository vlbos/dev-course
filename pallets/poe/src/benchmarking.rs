use crate::*;
#[allow(unused)]
use crate::Pallet as PoeModule;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::assert_ok;
use sp_std::vec;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_claim(c: Linear<0, { T::MaxClaimLength::get() }>) {
		let claim = BoundedVec::try_from(vec![]).unwrap();
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        create_claim(RawOrigin::Signed(caller.clone()), claim.clone());

        assert_eq!(
			Proofs::<T>::get(&claim),
			Some((caller.clone(), frame_system::Pallet::<T>::block_number()))
		);
        assert_last_event::<T>(Event::ClaimCreated(caller, claim).into())
    }
   
    #[benchmark]
    fn revoke_claim(c: Linear<0, { T::MaxClaimLength::get() }>) {
		let claim = BoundedVec::try_from(vec![0xff;c as usize]).unwrap();

        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Pallet::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone()));
        #[extrinsic_call]
        revoke_claim(RawOrigin::Signed(caller.clone()), claim.clone());

        assert_eq!(Proofs::<T>::get(&claim), None);
        assert_last_event::<T>(Event::ClaimRevoked(caller, claim).into())
    }

    #[benchmark]
    fn transfer_claim(c: Linear<0, { T::MaxClaimLength::get() }>) {
		let claim = BoundedVec::try_from(vec![0xff;c as usize]).unwrap();

        let caller: T::AccountId = whitelisted_caller();
        let target:T::AccountId=account("target",0,0);
        assert_ok!(Pallet::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone()));
        #[extrinsic_call]
        transfer_claim(RawOrigin::Signed(caller), claim.clone(),target.clone());

        assert_eq!(Proofs::<T>::get(&claim), Some((target, frame_system::Pallet::<T>::block_number())));
    }
    impl_benchmark_test_suite!(PoeModule, crate::mock::new_test_ext(), crate::mock::Test);
}