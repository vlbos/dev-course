use crate::{crypto, mock::*,PricePayload, Error, Event, Payload, SignedPayload, SigningTypes, Something};
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
fn should_submit_data_signed_transaction_on_chain() {
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
fn should_submit_data_unsigned_transaction_on_chain_for_any_account() {
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
fn should_submit_data_unsigned_transaction_on_chain_for_all_accounts() {
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
fn should_submit_data_raw_unsigned_transaction_on_chain() {
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = MemoryKeystore::new();

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt::new(keystore));

    // price_oracle_response(&mut offchain_state.write());

    t.execute_with(|| {
        let number = 42;
        // when
        TemplateModule::send_unsigned_tx(number).unwrap();
        // then
        let tx = pool_state.write().transactions.pop().unwrap();
        assert!(pool_state.read().transactions.is_empty());
        let tx = Extrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature, None);
        assert_eq!(
            tx.call,
            RuntimeCall::TemplateModule(crate::Call::submit_data_unsigned { number })
        );
    });
}




#[test]
fn it_aggregates_the_price() {
	sp_io::TestExternalities::default().execute_with(|| {
		assert_eq!(TemplateModule::average_price(), None);

		assert_ok!(TemplateModule::submit_price(RuntimeOrigin::signed(test_pub()), 27));
		assert_eq!(TemplateModule::average_price(), Some(27));

		assert_ok!(TemplateModule::submit_price(RuntimeOrigin::signed(test_pub()), 43));
		assert_eq!(TemplateModule::average_price(), Some(35));
	});
}

#[test]
fn should_make_http_call_and_parse_result() {
	let (offchain, state) = testing::TestOffchainExt::new();
	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));

	price_oracle_response(&mut state.write());

	t.execute_with(|| {
		// when
		let price = TemplateModule::fetch_price().unwrap();
		// then
		assert_eq!(price, 15523);
	});
}

#[test]
fn knows_how_to_mock_several_http_calls() {
	let (offchain, state) = testing::TestOffchainExt::new();
	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));

	{
		let mut state = state.write();
		state.expect_request(testing::PendingRequest {
			method: "GET".into(),
			uri: "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD".into(),
			response: Some(br#"{"USD": 1}"#.to_vec()),
			sent: true,
			..Default::default()
		});

		state.expect_request(testing::PendingRequest {
			method: "GET".into(),
			uri: "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD".into(),
			response: Some(br#"{"USD": 2}"#.to_vec()),
			sent: true,
			..Default::default()
		});

		state.expect_request(testing::PendingRequest {
			method: "GET".into(),
			uri: "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD".into(),
			response: Some(br#"{"USD": 3}"#.to_vec()),
			sent: true,
			..Default::default()
		});
	}

	t.execute_with(|| {
		let price1 = TemplateModule::fetch_price().unwrap();
		let price2 = TemplateModule::fetch_price().unwrap();
		let price3 = TemplateModule::fetch_price().unwrap();

		assert_eq!(price1, 100);
		assert_eq!(price2, 200);
		assert_eq!(price3, 300);
	})
}

#[test]
fn should_submit_signed_transaction_on_chain() {
	const PHRASE: &str =
		"news slush supreme milk chapter athlete soap sausage put clutch what kitten";

	let (offchain, offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();
	let keystore = MemoryKeystore::new();
	keystore
		.sr25519_generate_new(crate::crypto::Public::ID, Some(&format!("{}/hunter1", PHRASE)))
		.unwrap();

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt::new(keystore));

	price_oracle_response(&mut offchain_state.write());

	t.execute_with(|| {
		// when
		TemplateModule::fetch_price_and_send_signed().unwrap();
		// then
		let tx = pool_state.write().transactions.pop().unwrap();
		assert!(pool_state.read().transactions.is_empty());
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature.unwrap().0, 0);
		assert_eq!(tx.call, RuntimeCall::TemplateModule(crate::Call::submit_price { price: 15523 }));
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
		.sr25519_generate_new(crate::crypto::Public::ID, Some(&format!("{}/hunter1", PHRASE)))
		.unwrap();

	let public_key = *keystore.sr25519_public_keys(crate::crypto::Public::ID).get(0).unwrap();

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt::new(keystore));

	price_oracle_response(&mut offchain_state.write());

	let price_payload = PricePayload {
		block_number: 1,
		price: 15523,
		public: <Test as SigningTypes>::Public::from(public_key),
	};

	// let signature = price_payload.sign::<crypto::TestAuthId>().unwrap();
	t.execute_with(|| {
		// when
		TemplateModule::fetch_price_and_send_unsigned_for_any_account(1).unwrap();
		// then
		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		if let RuntimeCall::TemplateModule(crate::Call::submit_price_unsigned_with_signed_payload {
			price_payload: body,
			signature,
		}) = tx.call
		{
			assert_eq!(body, price_payload);

			let signature_valid =
				<PricePayload<
					<Test as SigningTypes>::Public,
					frame_system::pallet_prelude::BlockNumberFor<Test>,
				> as SignedPayload<Test>>::verify::<crypto::TestAuthId>(&price_payload, signature);

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
		.sr25519_generate_new(crate::crypto::Public::ID, Some(&format!("{}/hunter1", PHRASE)))
		.unwrap();

	let public_key = *keystore.sr25519_public_keys(crate::crypto::Public::ID).get(0).unwrap();

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt::new(keystore));

	price_oracle_response(&mut offchain_state.write());

	let price_payload = PricePayload {
		block_number: 1,
		price: 15523,
		public: <Test as SigningTypes>::Public::from(public_key),
	};

	// let signature = price_payload.sign::<crypto::TestAuthId>().unwrap();
	t.execute_with(|| {
		// when
		TemplateModule::fetch_price_and_send_unsigned_for_all_accounts(1).unwrap();
		// then
		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		if let RuntimeCall::TemplateModule(crate::Call::submit_price_unsigned_with_signed_payload {
			price_payload: body,
			signature,
		}) = tx.call
		{
			assert_eq!(body, price_payload);

			let signature_valid =
				<PricePayload<
					<Test as SigningTypes>::Public,
					frame_system::pallet_prelude::BlockNumberFor<Test>,
				> as SignedPayload<Test>>::verify::<crypto::TestAuthId>(&price_payload, signature);

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

	price_oracle_response(&mut offchain_state.write());

	t.execute_with(|| {
		// when
		TemplateModule::fetch_price_and_send_raw_unsigned(1).unwrap();
		// then
		let tx = pool_state.write().transactions.pop().unwrap();
		assert!(pool_state.read().transactions.is_empty());
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		assert_eq!(
			tx.call,
			RuntimeCall::TemplateModule(crate::Call::submit_price_unsigned {
				block_number: 1,
				price: 15523
			})
		);
	});
}

fn price_oracle_response(state: &mut testing::OffchainState) {
	state.expect_request(testing::PendingRequest {
		method: "GET".into(),
		uri: "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD".into(),
		response: Some(br#"{"USD": 155.23}"#.to_vec()),
		sent: true,
		..Default::default()
	});
}

#[test]
fn parse_price_works() {
	let test_data = alloc::vec![
		("{\"USD\":6536.92}", Some(653692)),
		("{\"USD\":65.92}", Some(6592)),
		("{\"USD\":6536.924565}", Some(653692)),
		("{\"USD\":6536}", Some(653600)),
		("{\"USD2\":6536}", None),
		("{\"USD\":\"6432\"}", None),
	];

	for (json, expected) in test_data {
		assert_eq!(expected, TemplateModule::parse_price(json));
	}
}
