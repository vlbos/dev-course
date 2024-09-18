use crate as pallet_kitties;
use frame_support::traits::Hooks;
use frame_support::{
    derive_impl,
    traits::{ConstU128, ConstU32, ConstU64},
    weights::Weight,
};
use sp_runtime::BuildStorage;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        PalletKitties: pallet_kitties,
        Random: pallet_insecure_randomness_collective_flip,
    }
);
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    // type BaseCallFilter = frame_support::traits::Everything;
    // type BlockWeights = ();
    // type BlockLength = ();
    // type DbWeight = ();
    // type RuntimeOrigin = RuntimeOrigin;
    // type RuntimeCall = RuntimeCall;
    // type Nonce = u64;
    // type Hash = H256;
    // type Hashing = BlakeTwo256;
    // type AccountId = u64;
    // type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    // type RuntimeEvent = RuntimeEvent;
    // type BlockHashCount = ConstU64<250>;
    // type Version = ();
    // type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    // type OnNewAccount = ();
    // type OnKilledAccount = ();
    // type SystemWeightInfo = ();
    // type SS58Prefix = ConstU16<42>;
    // type OnSetCode = ();
    // type MaxConsumers = frame_support::traits::ConstU32<16>;
}
#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    // type MaxLocks = ConstU32<50>;
    // type MaxReserves = ();
    // type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    // type RuntimeEvent = RuntimeEvent;
    // type DustRemoval = ();
    type ExistentialDeposit = ConstU128<500>;
    type AccountStore = System;
    // type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}
impl pallet_kitties::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Randomness = Random;
    type Currency = Balances;
    type StakeAmount = ConstU128<200>;
    type MinBidAmount = ConstU128<500>;
    type MinBidIncrement = ConstU128<500>;
    type MinBidBlockSpan = ConstU64<10>;
    type MaxKittiesBidPerBlock = ConstU32<10>;
}

impl pallet_insecure_randomness_collective_flip::Config for Test {}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 10_000_000_000),
            (2, 10_000_000_000),
            (3, 10_000_000_000),
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        log::info!("current block: {:?}", System::block_number());
        PalletKitties::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        PalletKitties::on_initialize(System::block_number());
        PalletKitties::on_idle(System::block_number(), Weight::default());
    }
}
