//! Autogenerated weights for pallet_merkle
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-02-17, STEPS: [20, ], REPEAT: 5, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: None, WASM-EXECUTION: Interpreted, CHAIN: Some("dev"), DB CACHE:
//! 128

// Executed Command:
// ./target/release/node-template
// benchmark
// --chain
// dev
// --pallet
// pallet_merkle
// --extrinsic
// *
// --steps
// 20
// --repeat
// 5
// --output
// ./pallets/merkle/src/

#![allow(unused_parens)]
#![allow(unused_imports)]

use crate::Config;
use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_merkle.
pub trait WeightInfo {
	fn create_group(n: u32) -> Weight;
	fn set_manager_required() -> Weight;
	fn set_manager() -> Weight;
	fn set_stopped() -> Weight;
	fn add_members(n: u32) -> Weight;
	fn verify_path(n: u32) -> Weight;
	fn on_finalize() -> Weight;
}

/// Weight functions for pallet_merkle.
pub struct Weights<T>(PhantomData<T>);
impl<T: frame_system::Config + Config> WeightInfo for Weights<T> {
	fn create_group(d: u32) -> Weight {
		(7_618_000 as Weight)
			// Standard Error: 4_000
			.saturating_add((151_000 as Weight).saturating_mul(d as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}

	fn set_manager_required() -> Weight {
		(8_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn set_manager() -> Weight {
		(8_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn set_stopped() -> Weight {
		(7_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn add_members(n: u32) -> Weight {
		(305_389_489_000 as Weight)
			// Standard Error: 4_552_643_000
			.saturating_add((63_659_275_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}

	fn verify_path(d: u32) -> Weight {
		(310_970_311_000 as Weight)
			// Standard Error: 673_763_000
			.saturating_add((3_666_683_000 as Weight).saturating_mul(d as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}

	fn on_finalize() -> Weight {
		(14_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
