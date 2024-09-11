use crate::{crypto, mock::*, Error, Event, Payload, SignedPayload, SigningTypes, Something};
use codec::Decode;
use frame_support::{assert_noop, assert_ok};
use sp_core::{
    offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
    sr25519::Signature,
    H256,
};

use sp_keystore::{testing::MemoryKeystore, Keystore, KeystoreExt};
use sp_runtime::{
    testing::TestXt,
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
    RuntimeAppPublic,
};
fn test_pub() -> sp_core::sr25519::Public {
    sp_core::sr25519::Public::from_raw([1u8; 32])
}
#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Go past genesis block so events get deposited
        System::set_block_number(1);
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::do_something(
            RuntimeOrigin::signed(test_pub()),
            42
        ));
        // Read pallet storage and assert an expected result.
        assert_eq!(Something::<Test>::get(), Some(42));
        // Assert that the correct event was deposited
        System::assert_last_event(
            Event::SomethingStored {
                something: 42,
                who: test_pub(),
            }
            .into(),
        );
    });
}

#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            TemplateModule::cause_error(RuntimeOrigin::signed(test_pub())),
            Error::<Test>::NoneValue
        );
    });
}

#[test]
fn should_submit_signed_transaction_on_chain() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";

    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();
    let keystore = MemoryKeystore::new();
    keystore
        .sr25519_generate_new(
            crate::crypto::Public::ID,
            Some(&format!("{}/hunter1", PHRASE)),
        )
        .unwrap();

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt::new(keystore));

    // price_oracle_response(&mut offchain_state.write());

    t.execute_with(|| {
        let payload = vec![3];
        // when
        TemplateModule::send_signed_tx(payload.clone()).unwrap();
        // then
        let tx = pool_state.write().transactions.pop().unwrap();
        assert!(pool_state.read().transactions.is_empty());
        let tx = Extrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature.unwrap().0, 0);
        assert_eq!(
            tx.call,
            RuntimeCall::TemplateModule(crate::Call::submit_data { payload })
        );
    });
}

#[test]
fn should_submit_unsigned_transaction_on_chain_for_any_account() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = MemoryKeystore::new();

    keystore
        .sr25519_generate_new(
            crate::crypto::Public::ID,
            Some(&format!("{}/hunter1", PHRASE)),
        )
        .unwrap();

    let public_key = *keystore
        .sr25519_public_keys(crate::crypto::Public::ID)
        .get(0)
        .unwrap();

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt::new(keystore));

    // price_oracle_response(&mut offchain_state.write());

    let payload = Payload {
        number: 1,
        public: <Test as SigningTypes>::Public::from(public_key),
    };

    // let signature = price_payload.sign::<crypto::TestAuthId>().unwrap();
    t.execute_with(|| {
        // when
        TemplateModule::send_unsigned_tx_with_payload(1).unwrap();
        // then
        let tx = pool_state.write().transactions.pop().unwrap();
        let tx = Extrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature, None);
        if let RuntimeCall::TemplateModule(crate::Call::unsigned_extrinsic_with_signed_payload {
            payload: body,
            signature,
        }) = tx.call
        {
            assert_eq!(body, payload);

            let signature_valid = <Payload<<Test as SigningTypes>::Public> as SignedPayload<
                Test,
            >>::verify::<crypto::TestAuthId>(&payload, signature);

            assert!(signature_valid);
        }
    });
}

#[test]
fn should_submit_unsigned_transaction_on_chain_for_all_accounts() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = MemoryKeystore::new();

    keystore
        .sr25519_generate_new(
            crate::crypto::Public::ID,
            Some(&format!("{}/hunter1", PHRASE)),
        )
        .unwrap();

    let public_key = *keystore
        .sr25519_public_keys(crate::crypto::Public::ID)
        .get(0)
        .unwrap();

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt::new(keystore));

    // price_oracle_response(&mut offchain_state.write());

    let payload = Payload {
        number: 1,
        public: <Test as SigningTypes>::Public::from(public_key),
    };

    // let signature = price_payload.sign::<crypto::TestAuthId>().unwrap();
    t.execute_with(|| {
        // when
        TemplateModule::send_unsigned_tx_with_payload_for_all_accounts(1).unwrap();
        // then
        let tx = pool_state.write().transactions.pop().unwrap();
        let tx = Extrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature, None);
        if let RuntimeCall::TemplateModule(crate::Call::unsigned_extrinsic_with_signed_payload {
            payload: body,
            signature,
        }) = tx.call
        {
            assert_eq!(body, payload);

            let signature_valid = <Payload<<Test as SigningTypes>::Public> as SignedPayload<
                Test,
            >>::verify::<crypto::TestAuthId>(&payload, signature);

            assert!(signature_valid);
        }
    });
}

#[test]
fn should_submit_raw_unsigned_transaction_on_chain() {
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = MemoryKeystore::new();

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt::new(keystore));

    // price_oracle_response(&mut offchain_state.write());

    t.execute_with(|| {
        let key = 42;
        // when
        TemplateModule::send_unsigned_tx(key).unwrap();
        // then
        let tx = pool_state.write().transactions.pop().unwrap();
        assert!(pool_state.read().transactions.is_empty());
        let tx = Extrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature, None);
        assert_eq!(
            tx.call,
            RuntimeCall::TemplateModule(crate::Call::submit_data_unsigned { key })
        );
    });
}
