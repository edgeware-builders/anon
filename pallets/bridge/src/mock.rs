use super::*;
use crate as webb_bridge;
use frame_benchmarking::whitelisted_caller;
use frame_support::{construct_runtime, parameter_types, weights::Weight, PalletId};
use frame_system::mocking::{MockBlock, MockUncheckedExtrinsic};
use pallet_merkle::weights::Weights as MerkleWeights;
use webb_currencies::BasicCurrencyAdapter;

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

pub(crate) type Balance = u64;
pub type Amount = i128;
pub type CurrencyId = u64;
pub type AccountId = u64;
pub type BlockNumber = u64;

// Configure a mock runtime to test the pallet.
type UncheckedExtrinsic = MockUncheckedExtrinsic<Test>;
type Block = MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Randomness: pallet_randomness_collective_flip::{Pallet, Call, Storage},
		MerkleTrees: pallet_merkle::{Pallet, Call, Storage, Event<T>},
		Bridge: webb_bridge::{Pallet, Call, Storage, Event<T>},
		Currencies: webb_currencies::{Pallet, Storage, Event<T>},
		Tokens: webb_tokens::{Pallet, Storage, Event<T>},
	}
);

parameter_types! {
	pub Prefix: u8 = 100;
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = AccountId;
	type BaseCallFilter = ();
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockNumber = BlockNumber;
	type BlockWeights = ();
	type Call = Call;
	type DbWeight = ();
	type Event = Event;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type Origin = Origin;
	type PalletInfo = PalletInfo;
	type SS58Prefix = Prefix;
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 0;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
	pub const MaxTreeDepth: u8 = 32;
	pub const CacheBlockLength: u64 = 5;
	// Minimum deposit length is 1 month w/ 6 second blocks
	pub const MinimumDepositLength: u64 = 10 * 60 * 24 * 28;
}

impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

parameter_types! {
	pub const TokensPalletId: PalletId = PalletId(*b"py/token");
	pub const CurrencyDeposit: u64 = 0;
	pub const ApprovalDeposit: u64 = 1;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u64 = 1;
	pub const MetadataDepositPerByte: u64 = 1;
}

parameter_types! {
	pub DustAccount: AccountId = PalletId(*b"webb/dst").into_account();
}

impl webb_tokens::Config for Test {
	type Amount = i128;
	type ApprovalDeposit = ApprovalDeposit;
	type Balance = Balance;
	type CurrencyDeposit = CurrencyDeposit;
	type CurrencyId = CurrencyId;
	type DustAccount = DustAccount;
	type Event = Event;
	type Extra = ();
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type NativeCurrency = BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;
	type PalletId = TokensPalletId;
	type StringLimit = StringLimit;
	type WeightInfo = ();
}

impl webb_currencies::Config for Test {
	type Event = Event;
	type GetNativeCurrencyId = NativeCurrencyId;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;
	type WeightInfo = ();
}

impl pallet_merkle::Config for Test {
	type CacheBlockLength = CacheBlockLength;
	type Event = Event;
	type KeyId = u32;
	type MaxTreeDepth = MaxTreeDepth;
	type Randomness = Randomness;
	type TreeId = u32;
	type WeightInfo = MerkleWeights<Self>;
}

parameter_types! {
	pub const BridgePalletId: PalletId = PalletId(*b"py/brdge");
	pub const DefaultAdmin: u64 = 4;
	pub const NativeCurrencyId: CurrencyId = 0;
}

impl Config for Test {
	type ChainId = u32;
	type Currency = Tokens;
	type DefaultAdmin = DefaultAdmin;
	type Event = Event;
	type NativeCurrencyId = NativeCurrencyId;
	type PalletId = BridgePalletId;
	type ThresholdSignature = [u8; 32];
	type Tree = MerkleTrees;
}

impl pallet_randomness_collective_flip::Config for Test {}

pub type TokenPallet = webb_tokens::Pallet<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	use pallet_balances::GenesisConfig as BalancesConfig;
	// use tokens::GenesisConfig as TokensConfig;
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	BalancesConfig::<Test> {
		// Total issuance will be 200 with treasury account initialized at ED.
		balances: vec![
			(0, 1_000_000_000_000_000_000),
			(1, 1_000_000_000_000_000_000),
			(2, 1_000_000_000_000_000_000),
			(whitelisted_caller(), 1_000_000_000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t.into()
}
