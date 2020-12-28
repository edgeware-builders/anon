use bulletproofs::r1cs::LinearCombination;
use rand::rngs::ThreadRng;
use sp_runtime::DispatchError;
use curve25519_dalek::scalar::Scalar;
use merkle::merkle::hasher::Hasher;
use merkle::merkle::helper::{commit_leaf, commit_path_level, leaf_data};
use merkle::merkle::keys::{Commitment, Data};
use merkle::merkle::poseidon::Poseidon;
use crate::mock::*;
use bulletproofs::r1cs::{ConstraintSystem, Prover};
use bulletproofs::{BulletproofGens, PedersenGens};

use frame_support::{assert_err, assert_ok};
use merlin::Transcript;

fn key_bytes(x: u8) -> [u8; 32] {
	[
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, x,
	]
}

fn default_hasher() -> impl Hasher {
	Poseidon::new(4)
	// Mimc::new(70)
}

fn create_deposit_info(mut test_rng: &mut ThreadRng) -> (Scalar, Scalar, Data) {
	let h = default_hasher();
	let (s, nullifier, leaf) = leaf_data(&mut test_rng, &h);
	(s, nullifier, leaf)
}

#[test]
fn should_initialize_successfully() {
	new_test_ext().execute_with(|| {
		assert_ok!(Mixer::initialize(Origin::signed(1)));
		// the mixer creates 4 groups, they should all initialise to 0
		let val = 1_000;
		for i in 0..4 {
			let g = MerkleGroups::get_group(i).unwrap();
			let m = Mixer::get_mixer(i).unwrap();
			assert_eq!(g.leaf_count, 0);
			assert_eq!(g.manager_required, true);
			assert_eq!(m.leaves.len(), 0);
			assert_eq!(m.fixed_deposit_size, val * 10_u64.pow(i))
		}
	})
}

#[test]
fn should_fail_to_deposit_with_insufficient_balance() {
	new_test_ext().execute_with(|| {
		assert_ok!(Mixer::initialize(Origin::signed(1)));
		let mut test_rng = rand::thread_rng();
		let mut deposits = vec![];
		for i in 0..4 {
			let dep = create_deposit_info(&mut test_rng);
			deposits.push(dep);
			// ensure depositing works
			let (_, _, leaf) = dep;
			assert_err!(
				Mixer::deposit(Origin::signed(4), i, vec![leaf]),
				DispatchError::Module {
					index: 0,
					error: 6,
					message: Some("InsufficientBalance")
				}
			);
		}
	})
}

#[test]
fn should_deposit_into_each_mixer_successfully() {
	new_test_ext().execute_with(|| {
		assert_ok!(Mixer::initialize(Origin::signed(1)));
		let mut deposits = vec![];
		let mut test_rng = rand::thread_rng();
		for i in 0..4 {
			let dep = create_deposit_info(&mut test_rng);
			deposits.push(dep);
			// ensure depositing works
			let (_, _, leaf) = dep;
			let balance_before = Balances::free_balance(1);
			assert_ok!(Mixer::deposit(Origin::signed(1), i, vec![leaf]));
			let balance_after = Balances::free_balance(1);

			// ensure state updates
			let g = MerkleGroups::get_group(i).unwrap();
			let m = Mixer::get_mixer(i).unwrap();
			assert_eq!(balance_before, balance_after + m.fixed_deposit_size);
			assert_eq!(g.leaf_count, 1);
			assert_eq!(m.leaves.len(), 1);
		}
	})
}

#[test]
fn should_withdraw_from_each_mixer_successfully() {
	new_test_ext().execute_with(|| {
		assert_ok!(Mixer::initialize(Origin::signed(1)));
		let mut test_rng = rand::thread_rng();
		let h = default_hasher();
		let pc_gens = PedersenGens::default();
		let bp_gens = BulletproofGens::new(4096, 1);

		let mut deposits = vec![];
		for i in 0..4 {
			let dep = create_deposit_info(&mut test_rng);
			deposits.push(dep);
			// ensure depositing works
			let (s, nullifier, leaf) = dep;
			assert_ok!(Mixer::deposit(Origin::signed(1), i, vec![leaf]));

			let root = MerkleGroups::get_merkle_root(i);
			let mut prover_transcript = Transcript::new(b"zk_membership_proof");
			let mut prover = Prover::new(&pc_gens, &mut prover_transcript);

			let (s_com, leaf_com1, leaf_var1) =
				commit_leaf(&mut test_rng, &mut prover, leaf, s, nullifier, &h);

			let mut lh = leaf;
			let mut lh_lc: LinearCombination = leaf_var1.into();
			let mut path = Vec::new();
			for _ in 0..32 {
				let (bit_com, leaf_com, node_con) =
					commit_path_level(&mut test_rng, &mut prover, lh, lh_lc, 1, &h);
				lh_lc = node_con;
				lh = Data::hash(lh, lh, &h);
				path.push((Commitment(bit_com), Commitment(leaf_com)));
			}
			prover.constrain(lh_lc - lh.0);

			let proof = prover.prove_with_rng(&bp_gens, &mut test_rng).unwrap();

			let m = Mixer::get_mixer(i).unwrap();
			let balance_before = Balances::free_balance(2);
			// withdraw from another account
			assert_ok!(Mixer::withdraw(
				Origin::signed(2),
				i,
				0,
				root.unwrap(),
				Commitment(leaf_com1),
				path,
				Commitment(s_com),
				Data(nullifier),
				proof.to_bytes(),
			));
			let balance_after = Balances::free_balance(2);
			assert_eq!(balance_before + m.fixed_deposit_size, balance_after);
		}
	})
}