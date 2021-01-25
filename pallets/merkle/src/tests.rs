use super::*;
use crate::{
	merkle::keys::{Commitment, Data},
	mock::*,
};
use bulletproofs::{r1cs::Prover, BulletproofGens, PedersenGens};
use curve25519_dalek::{ristretto::RistrettoPoint, scalar::Scalar};
use curve25519_gadgets::{
	crypto_constants::smt::ZERO_TREE,
	fixed_deposit_tree::builder::FixedDepositTreeBuilder,
	poseidon::{
		builder::{gen_round_keys, Poseidon, PoseidonBuilder},
		PoseidonSbox, Poseidon_hash_2,
	},
};
use frame_support::{assert_err, assert_ok};
use merlin::Transcript;
use rand_core::OsRng;

fn key_bytes(x: u8) -> [u8; 32] {
	[
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, x,
	]
}

fn default_hasher(num_gens: usize) -> Poseidon {
	let width = 6;
	let (full_b, full_e) = (4, 4);
	let partial_rounds = 57;
	PoseidonBuilder::new(width)
		.num_rounds(full_b, full_e, partial_rounds)
		.round_keys(gen_round_keys(width, full_b + full_e + partial_rounds))
		.mds_matrix(gen_mds_matrix(width))
		.bulletproof_gens(BulletproofGens::new(num_gens, 1))
		.sbox(PoseidonSbox::Inverse)
		.build()
}

#[test]
fn can_create_group() {
	new_test_ext().execute_with(|| {
		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(3),));
	});
}

#[test]
fn can_update_manager_when_required() {
	new_test_ext().execute_with(|| {
		assert_ok!(MerkleGroups::create_group(Origin::signed(1), true, Some(3),));

		assert_ok!(MerkleGroups::set_manager(Origin::signed(1), 0, 2,));
	});
}

#[test]
fn can_update_manager_when_not_required() {
	new_test_ext().execute_with(|| {
		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(3),));

		assert_ok!(MerkleGroups::set_manager(Origin::signed(1), 0, 2,));
	});
}

#[test]
fn cannot_update_manager_as_not_manager() {
	new_test_ext().execute_with(|| {
		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(3),));

		assert_err!(
			MerkleGroups::set_manager(Origin::signed(2), 0, 2,),
			Error::<Test>::ManagerIsRequired
		);
	});
}

#[test]
fn can_update_manager_required_manager() {
	new_test_ext().execute_with(|| {
		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(3),));

		assert_ok!(MerkleGroups::set_manager_required(Origin::signed(1), 0, true,));
	});
}

#[test]
fn cannot_update_manager_required_as_not_manager() {
	new_test_ext().execute_with(|| {
		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(3),));

		assert_err!(
			MerkleGroups::set_manager_required(Origin::signed(2), 0, true,),
			Error::<Test>::ManagerIsRequired
		);
	});
}

#[test]
fn can_add_member() {
	new_test_ext().execute_with(|| {
		let key = Data::from(key_bytes(1));

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(3),));
		assert_ok!(MerkleGroups::add_members(Origin::signed(1), 0, vec![key.clone()]));
	});
}

#[test]
fn can_add_member_as_manager() {
	new_test_ext().execute_with(|| {
		let key = Data::from(key_bytes(1));

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), true, Some(3),));
		assert_ok!(MerkleGroups::add_members(Origin::signed(1), 0, vec![key.clone()]));
	});
}

#[test]
fn cannot_add_member_as_not_manager() {
	new_test_ext().execute_with(|| {
		let key = Data::from(key_bytes(1));

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), true, Some(3),));
		assert_err!(
			MerkleGroups::add_members(Origin::signed(2), 0, vec![key.clone()]),
			Error::<Test>::ManagerIsRequired
		);
	});
}

#[test]
fn should_not_have_0_depth() {
	new_test_ext().execute_with(|| {
		assert_err!(
			MerkleGroups::create_group(Origin::signed(1), false, Some(0)),
			Error::<Test>::InvalidTreeDepth,
		);
	});
}

#[test]
fn should_have_min_depth() {
	new_test_ext().execute_with(|| {
		let key = Data::from(key_bytes(1));
		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(1),));

		assert_ok!(MerkleGroups::add_members(Origin::signed(1), 0, vec![key.clone()]));
		assert_err!(
			MerkleGroups::add_members(Origin::signed(1), 0, vec![key.clone()]),
			Error::<Test>::ExceedsMaxDepth,
		);
	});
}

#[test]
fn should_have_max_depth() {
	new_test_ext().execute_with(|| {
		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(32),));
	});
}

#[test]
fn should_not_have_more_than_max_depth() {
	new_test_ext().execute_with(|| {
		assert_err!(
			MerkleGroups::create_group(Origin::signed(1), false, Some(33),),
			Error::<Test>::InvalidTreeDepth,
		);
	});
}

#[test]
fn should_have_correct_root_hash_after_insertion() {
	new_test_ext().execute_with(|| {
		let h = default_hasher(4096);
		let key0 = Data::from(key_bytes(0));
		let key1 = Data::from(key_bytes(1));
		let key2 = Data::from(key_bytes(2));
		let zero_h0 = Data::from(ZERO_TREE[0]);
		let zero_h1 = Data::from(ZERO_TREE[1]);

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(2),));
		assert_ok!(MerkleGroups::add_members(Origin::signed(1), 0, vec![key0.clone()]));

		let keyh1 = Poseidon_hash_2(key0.0, zero_h0.0, &h);
		let keyh2 = Poseidon_hash_2(keyh1, zero_h1.0, &h);

		let tree = MerkleGroups::groups(0).unwrap();

		assert_eq!(tree.root_hash.0, keyh2, "Invalid root hash");

		assert_ok!(MerkleGroups::add_members(Origin::signed(2), 0, vec![key1.clone()]));

		let keyh1 = Poseidon_hash_2(key0.0, key1.0, &h);
		let keyh2 = Poseidon_hash_2(keyh1, zero_h1.0, &h);

		let tree = MerkleGroups::groups(0).unwrap();

		assert_eq!(tree.root_hash.0, keyh2, "Invalid root hash");

		assert_ok!(MerkleGroups::add_members(Origin::signed(3), 0, vec![key2.clone()]));

		let keyh1 = Poseidon_hash_2(key0.0, key1.0, &h);
		let keyh2 = Poseidon_hash_2(key2.0, zero_h0.0, &h);
		let keyh3 = Poseidon_hash_2(keyh1, keyh2, &h);

		let tree = MerkleGroups::groups(0).unwrap();

		assert_eq!(tree.root_hash.0, keyh3, "Invalid root hash");
	});
}

#[test]
fn should_have_correct_root_hash() {
	new_test_ext().execute_with(|| {
		let h = default_hasher(4096);
		let mut keys = Vec::new();
		for i in 0..15 {
			keys.push(Scalar::from_bytes_mod_order(key_bytes(i as u8)))
		}
		let zero_h0 = Data::from(ZERO_TREE[0]);

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(4),));
		let keys_data: Vec<Data> = keys.iter().map(|x| Data(*x)).collect();
		assert_ok!(MerkleGroups::add_members(Origin::signed(0), 0, keys_data.clone()));

		let key1_1 = Poseidon_hash_2(keys[0], keys[1], &h);
		let key1_2 = Poseidon_hash_2(keys[2], keys[3], &h);
		let key1_3 = Poseidon_hash_2(keys[4], keys[5], &h);
		let key1_4 = Poseidon_hash_2(keys[6], keys[7], &h);
		let key1_5 = Poseidon_hash_2(keys[8], keys[9], &h);
		let key1_6 = Poseidon_hash_2(keys[10], keys[11], &h);
		let key1_7 = Poseidon_hash_2(keys[12], keys[13], &h);
		let key1_8 = Poseidon_hash_2(keys[14], zero_h0.0, &h);

		let key2_1 = Poseidon_hash_2(key1_1, key1_2, &h);
		let key2_2 = Poseidon_hash_2(key1_3, key1_4, &h);
		let key2_3 = Poseidon_hash_2(key1_5, key1_6, &h);
		let key2_4 = Poseidon_hash_2(key1_7, key1_8, &h);

		let key3_1 = Poseidon_hash_2(key2_1, key2_2, &h);
		let key3_2 = Poseidon_hash_2(key2_3, key2_4, &h);

		let root_hash = Poseidon_hash_2(key3_1, key3_2, &h);

		let tree = MerkleGroups::groups(0).unwrap();

		assert_eq!(tree.root_hash.0, root_hash, "Invalid root hash");
	});
}

#[test]
fn should_be_unable_to_pass_proof_path_with_invalid_length() {
	new_test_ext().execute_with(|| {
		let key0 = Data::from(key_bytes(0));
		let key1 = Data::from(key_bytes(1));
		let key2 = Data::from(key_bytes(2));
		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(2),));
		assert_ok!(MerkleGroups::add_members(Origin::signed(0), 0, vec![
			key0.clone(),
			key1.clone(),
			key2.clone()
		]));

		let path = vec![(true, key0)];
		assert_err!(
			MerkleGroups::verify(Origin::signed(2), 0, key0, path),
			Error::<Test>::InvalidPathLength,
		);

		let path = vec![(true, key0), (false, key1), (true, key2)];
		assert_err!(
			MerkleGroups::verify(Origin::signed(2), 0, key0, path),
			Error::<Test>::InvalidPathLength,
		);
	});
}

#[test]
fn should_not_verify_invalid_proof() {
	new_test_ext().execute_with(|| {
		let h = default_hasher(4096);
		let key0 = Data::from(key_bytes(9));
		let key1 = Data::from(key_bytes(3));
		let key2 = Data::from(key_bytes(5));
		let zero_h0 = Data::from(ZERO_TREE[0]);

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(2),));
		assert_ok!(MerkleGroups::add_members(Origin::signed(1), 0, vec![
			key0.clone(),
			key1.clone(),
			key2.clone()
		]));

		let keyh1 = Poseidon_hash_2(key0.0, key1.0, &h);
		let keyh2 = Poseidon_hash_2(key2.0, zero_h0.0, &h);
		let _root_hash = Poseidon_hash_2(keyh1, keyh2, &h);

		let path = vec![(false, key1), (true, Data(keyh2))];

		assert_err!(
			MerkleGroups::verify(Origin::signed(2), 0, key0, path),
			Error::<Test>::InvalidMembershipProof,
		);

		let path = vec![(true, key1), (false, Data(keyh2))];

		assert_err!(
			MerkleGroups::verify(Origin::signed(2), 0, key0, path),
			Error::<Test>::InvalidMembershipProof,
		);

		let path = vec![(true, key2), (true, Data(keyh1))];

		assert_err!(
			MerkleGroups::verify(Origin::signed(2), 0, key0, path),
			Error::<Test>::InvalidMembershipProof,
		);
	});
}

#[test]
fn should_verify_proof_of_membership() {
	new_test_ext().execute_with(|| {
		let h = default_hasher(4096);
		let mut keys = Vec::new();
		for i in 0..15 {
			keys.push(Scalar::from_bytes_mod_order(key_bytes(i as u8)))
		}
		let zero_h0 = Data::from(ZERO_TREE[0]);

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(4),));
		let keys_data: Vec<Data> = keys.iter().map(|x| Data(*x)).collect();
		assert_ok!(MerkleGroups::add_members(Origin::signed(0), 0, keys_data.clone()));

		let key1_1 = Poseidon_hash_2(keys[0], keys[1], &h);
		let key1_2 = Poseidon_hash_2(keys[2], keys[3], &h);
		let key1_3 = Poseidon_hash_2(keys[4], keys[5], &h);
		let key1_4 = Poseidon_hash_2(keys[6], keys[7], &h);
		let key1_5 = Poseidon_hash_2(keys[8], keys[9], &h);
		let key1_6 = Poseidon_hash_2(keys[10], keys[11], &h);
		let key1_7 = Poseidon_hash_2(keys[12], keys[13], &h);
		let key1_8 = Poseidon_hash_2(keys[14], zero_h0.0, &h);

		let key2_1 = Poseidon_hash_2(key1_1, key1_2, &h);
		let key2_2 = Poseidon_hash_2(key1_3, key1_4, &h);
		let key2_3 = Poseidon_hash_2(key1_5, key1_6, &h);
		let key2_4 = Poseidon_hash_2(key1_7, key1_8, &h);

		let key3_1 = Poseidon_hash_2(key2_1, key2_2, &h);
		let key3_2 = Poseidon_hash_2(key2_3, key2_4, &h);

		let _root_hash = Poseidon_hash_2(key3_1, key3_2, &h);

		let path = vec![
			(true, keys_data[1]),
			(true, Data(key1_2)),
			(true, Data(key2_2)),
			(true, Data(key3_2)),
		];

		assert_ok!(MerkleGroups::verify(Origin::signed(2), 0, keys_data[0], path));

		let path = vec![
			(true, keys_data[5]),
			(true, Data(key1_4)),
			(false, Data(key2_1)),
			(true, Data(key3_2)),
		];

		assert_ok!(MerkleGroups::verify(Origin::signed(2), 0, keys_data[4], path));

		let path = vec![
			(true, keys_data[11]),
			(false, Data(key1_5)),
			(true, Data(key2_4)),
			(false, Data(key3_1)),
		];

		assert_ok!(MerkleGroups::verify(Origin::signed(2), 0, keys_data[10], path));

		let path = vec![
			(true, zero_h0),
			(false, Data(key1_7)),
			(false, Data(key2_3)),
			(false, Data(key3_1)),
		];

		assert_ok!(MerkleGroups::verify(Origin::signed(2), 0, keys_data[14], path));
	});
}

#[test]
fn should_verify_simple_zk_proof_of_membership() {
	new_test_ext().execute_with(|| {
		let pc_gens = PedersenGens::default();

		let label = b"zk_membership_proof";
		let mut prover_transcript = Transcript::new(label);
		let prover = Prover::new(&pc_gens, &mut prover_transcript);

		let mut ftree = FixedDepositTreeBuilder::new().depth(1).build();

		let leaf = ftree.add_secrets();
		ftree.tree.add(vec![leaf]);

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(1),));
		assert_ok!(MerkleGroups::add_members(Origin::signed(1), 0, vec![Data(leaf)]));
		let root = MerkleGroups::get_merkle_root(0).unwrap();

		let (proof, (comms_cr, nullifier_hash, leaf_index_comms_cr, proof_comms_cr)) =
			ftree.prove_zk(Scalar::zero(), root.0, &ftree.hash_params.bp_gens, prover);

		let comms: Vec<Commitment> = comms_cr.iter().map(|x| Commitment(*x)).collect();
		let leaf_index_comms: Vec<Commitment> = leaf_index_comms_cr.iter().map(|x| Commitment(*x)).collect();
		let proof_comms: Vec<Commitment> = proof_comms_cr.iter().map(|x| Commitment(*x)).collect();
		assert_ok!(MerkleGroups::verify_zk_membership_proof(
			0,
			0,
			root,
			comms,
			Data(nullifier_hash),
			proof.to_bytes(),
			leaf_index_comms,
			proof_comms
		));
	});
}

#[test]
fn should_not_verify_invalid_commitments_for_leaf_creation() {
	new_test_ext().execute_with(|| {
		let pc_gens = PedersenGens::default();

		let label = b"zk_membership_proof";
		let mut prover_transcript = Transcript::new(label);
		let prover = Prover::new(&pc_gens, &mut prover_transcript);

		let mut ftree = FixedDepositTreeBuilder::new().depth(1).build();

		let leaf = ftree.add_secrets();
		ftree.tree.add(vec![leaf]);

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(1),));
		assert_ok!(MerkleGroups::add_members(Origin::signed(1), 0, vec![Data(leaf)]));
		let root = MerkleGroups::get_merkle_root(0).unwrap();

		let (proof, (comms_cr, nullifier_hash, leaf_index_comms_cr, proof_comms_cr)) =
			ftree.prove_zk(Scalar::zero(), root.0, &ftree.hash_params.bp_gens, prover);

		let mut comms: Vec<Commitment> = comms_cr.iter().map(|x| Commitment(*x)).collect();
		let mut rng = OsRng::default();
		comms[0] = Commitment(RistrettoPoint::random(&mut rng).compress());
		let leaf_index_comms: Vec<Commitment> = leaf_index_comms_cr.iter().map(|x| Commitment(*x)).collect();
		let proof_comms: Vec<Commitment> = proof_comms_cr.iter().map(|x| Commitment(*x)).collect();
		assert_err!(
			MerkleGroups::verify_zk_membership_proof(
				0,
				0,
				root,
				comms,
				Data(nullifier_hash),
				proof.to_bytes(),
				leaf_index_comms,
				proof_comms
			),
			Error::<Test>::ZkVericationFailed
		);
	});
}

#[test]
fn should_not_verify_invalid_commitments_for_membership() {
	new_test_ext().execute_with(|| {
		let pc_gens = PedersenGens::default();

		let label = b"zk_membership_proof";
		let mut prover_transcript = Transcript::new(label);
		let prover = Prover::new(&pc_gens, &mut prover_transcript);

		let mut ftree = FixedDepositTreeBuilder::new().depth(1).build();

		let leaf = ftree.add_secrets();
		ftree.tree.add(vec![leaf]);

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(1),));
		assert_ok!(MerkleGroups::add_members(Origin::signed(1), 0, vec![Data(leaf)]));
		let root = MerkleGroups::get_merkle_root(0).unwrap();

		let (proof, (comms_cr, nullifier_hash, leaf_index_comms_cr, proof_comms_cr)) =
			ftree.prove_zk(Scalar::zero(), root.0, &ftree.hash_params.bp_gens, prover);

		let comms: Vec<Commitment> = comms_cr.iter().map(|x| Commitment(*x)).collect();
		let mut leaf_index_comms: Vec<Commitment> = leaf_index_comms_cr.iter().map(|x| Commitment(*x)).collect();
		let mut proof_comms: Vec<Commitment> = proof_comms_cr.iter().map(|x| Commitment(*x)).collect();
		let mut rng = OsRng::default();
		leaf_index_comms[0] = Commitment(RistrettoPoint::random(&mut rng).compress());
		proof_comms[0] = Commitment(RistrettoPoint::random(&mut rng).compress());
		assert_err!(
			MerkleGroups::verify_zk_membership_proof(
				0,
				0,
				root,
				comms,
				Data(nullifier_hash),
				proof.to_bytes(),
				leaf_index_comms,
				proof_comms
			),
			Error::<Test>::ZkVericationFailed
		);
	});
}

#[test]
fn should_not_verify_invalid_transcript() {
	new_test_ext().execute_with(|| {
		let pc_gens = PedersenGens::default();

		let label = b"zk_membership_proof_invalid";
		let mut prover_transcript = Transcript::new(label);
		let prover = Prover::new(&pc_gens, &mut prover_transcript);

		let mut ftree = FixedDepositTreeBuilder::new().depth(1).build();

		let leaf = ftree.add_secrets();
		ftree.tree.add(vec![leaf]);

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(1),));
		assert_ok!(MerkleGroups::add_members(Origin::signed(1), 0, vec![Data(leaf)]));
		let root = MerkleGroups::get_merkle_root(0).unwrap();

		let (proof, (comms_cr, nullifier_hash, leaf_index_comms_cr, proof_comms_cr)) =
			ftree.prove_zk(Scalar::zero(), root.0, &ftree.hash_params.bp_gens, prover);

		let comms: Vec<Commitment> = comms_cr.iter().map(|x| Commitment(*x)).collect();
		let leaf_index_comms: Vec<Commitment> = leaf_index_comms_cr.iter().map(|x| Commitment(*x)).collect();
		let proof_comms: Vec<Commitment> = proof_comms_cr.iter().map(|x| Commitment(*x)).collect();
		assert_err!(
			MerkleGroups::verify_zk_membership_proof(
				0,
				0,
				root,
				comms,
				Data(nullifier_hash),
				proof.to_bytes(),
				leaf_index_comms,
				proof_comms
			),
			Error::<Test>::ZkVericationFailed
		);
	});
}

#[test]
fn should_verify_zk_proof_of_membership() {
	new_test_ext().execute_with(|| {
		let pc_gens = PedersenGens::default();

		let mut prover_transcript = Transcript::new(b"zk_membership_proof");
		let prover = Prover::new(&pc_gens, &mut prover_transcript);
		let mut ftree = FixedDepositTreeBuilder::new().depth(3).build();

		let leaf0 = ftree.add_secrets();
		let leaf1 = ftree.add_secrets();
		let leaf2 = ftree.add_secrets();
		let leaf3 = ftree.add_secrets();
		let leaf4 = ftree.add_secrets();
		let leaf5 = ftree.add_secrets();
		let leaf6 = ftree.add_secrets();
		let keys = vec![leaf0, leaf1, leaf2, leaf3, leaf4, leaf5, leaf6];
		ftree.tree.add(keys.clone());

		let keys_data: Vec<Data> = keys.iter().map(|x| Data(*x)).collect();
		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(3),));
		assert_ok!(MerkleGroups::add_members(Origin::signed(1), 0, keys_data));

		let root = MerkleGroups::get_merkle_root(0).unwrap();
		let (proof, (comms_cr, nullifier_hash, leaf_index_comms_cr, proof_comms_cr)) =
			ftree.prove_zk(Scalar::from(5u8), root.0, &ftree.hash_params.bp_gens, prover);

		let comms: Vec<Commitment> = comms_cr.iter().map(|x| Commitment(*x)).collect();
		let leaf_index_comms: Vec<Commitment> = leaf_index_comms_cr.iter().map(|x| Commitment(*x)).collect();
		let proof_comms: Vec<Commitment> = proof_comms_cr.iter().map(|x| Commitment(*x)).collect();
		assert_ok!(MerkleGroups::verify_zk_membership_proof(
			0,
			0,
			root,
			comms,
			Data(nullifier_hash),
			proof.to_bytes(),
			leaf_index_comms,
			proof_comms
		));
	});
}

#[test]
fn should_verify_large_zk_proof_of_membership() {
	new_test_ext().execute_with(|| {
		let pc_gens = PedersenGens::default();

		let mut prover_transcript = Transcript::new(b"zk_membership_proof");
		let prover = Prover::new(&pc_gens, &mut prover_transcript);
		let poseidon = default_hasher(40960);
		let mut ftree = FixedDepositTreeBuilder::new().hash_params(poseidon).depth(32).build();

		let leaf = ftree.add_secrets();
		ftree.tree.add(vec![leaf]);

		assert_ok!(MerkleGroups::create_group(Origin::signed(1), false, Some(32),));
		assert_ok!(MerkleGroups::add_members(Origin::signed(1), 0, vec![Data(leaf)]));

		let root = MerkleGroups::get_merkle_root(0).unwrap();
		let (proof, (comms_cr, nullifier_hash, leaf_index_comms_cr, proof_comms_cr)) =
			ftree.prove_zk(Scalar::zero(), root.0, &ftree.hash_params.bp_gens, prover);

		let comms: Vec<Commitment> = comms_cr.iter().map(|x| Commitment(*x)).collect();
		let leaf_index_comms: Vec<Commitment> = leaf_index_comms_cr.iter().map(|x| Commitment(*x)).collect();
		let proof_comms: Vec<Commitment> = proof_comms_cr.iter().map(|x| Commitment(*x)).collect();
		assert_ok!(MerkleGroups::verify_zk_membership_proof(
			0,
			0,
			root,
			comms,
			Data(nullifier_hash),
			proof.to_bytes(),
			leaf_index_comms,
			proof_comms
		));
	});
}
