use crate::constants::BASEPOINT;
use sha2::Sha512;
use curve25519_dalek::ristretto::RistrettoPoint;
use codec::alloc::collections::HashSet;
use codec::{Encode, Decode, Input, EncodeLike};
use curve25519_dalek::ristretto::{
	CompressedRistretto,
	// RistrettoPoint
};
use curve25519_dalek::scalar::Scalar;


#[derive(Eq, PartialEq, Clone, Default, Debug)]
pub struct RingPublicKey(pub CompressedRistretto);


pub const SIZE: usize = 32;

impl Encode for RingPublicKey {
    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        (self.0).0.using_encoded(f)
    }
}

impl EncodeLike for RingPublicKey {}

impl Decode for RingPublicKey {
    fn decode<I: Input>(input: &mut I) -> Result<Self, codec::Error> {
        match <[u8; SIZE] as Decode>::decode(input).map(CompressedRistretto) {
        	Ok(elt) => Ok(RingPublicKey(elt)),
        	Err(e) => Err(e),
        }
    }
}

impl RingPublicKey {
	/// Constructor from bytes
	pub fn new(bytes: [u8; 32]) -> Self {
		RingPublicKey(CompressedRistretto(bytes))
	}
    /// Serialize this public key to 32 bytes
    pub fn as_bytes(&self) -> Vec<u8> {
        (&self.0.as_bytes()).to_vec()
    }

    // TODO: Make this more robust
    /// Deserialize this public key from 32 bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<RingPublicKey> {
        if bytes.len() != 32 {
            return None;
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        let c = CompressedRistretto(arr);
        Some(RingPublicKey(c))
    }
}

#[derive(Debug, Clone)]
pub struct PublicSet(pub Vec<RistrettoPoint>);

impl PublicSet {
    // Returns the number of public keys in the set
    pub fn len(&self) -> usize {
        self.0.len()
    }
    // Checks if the public set contains any duplicate keys
    pub fn duplicates_exist(&self) -> bool {
        // XXX: Very in-efficient way to do this.
        // We can wait for upstream crate to implement Hash and use a HashSet instead

        let compressed_points: Vec<CompressedRistretto> =
            self.0.iter().map(|point| point.compress()).collect();

        let hashable_slice: Vec<&[u8; 32]> =
            compressed_points.iter().map(|cp| cp.as_bytes()).collect();

        let uniques: HashSet<_> = hashable_slice.iter().collect();

        self.0.len() != uniques.len()
    }
    // Returns the Hash_to_point of the first public key in the set
    // This point is used extensively during the protocol for each member
    pub fn hashed_pubkey(&self) -> RistrettoPoint {
        let first_pubkey = &self.0[0].compress();
        RistrettoPoint::hash_from_bytes::<Sha512>(first_pubkey.as_bytes())
    }
    // Copies the public key set into a vector of bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0
            .iter()
            .map(|point| point.compress().to_bytes().to_vec())
            .flatten()
            .collect()
    }

    pub fn to_keys(&self) -> Vec<CompressedRistretto> {
        self.0.iter().map(|key| key.compress()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct PrivateSet(pub(crate) Vec<Scalar>);

impl PrivateSet {
    pub fn new(scalars: Vec<Scalar>) -> Self {
        PrivateSet(scalars)
    }
    // Takes a set of private keys
    // and returns the corresponding public key set
    // along with the basepoint used in calculating the key images
    pub fn to_public_set(&self) -> PublicSet {
        let public_keys = self
            .0
            .iter()
            .map(|&x| x * BASEPOINT)
            .collect::<Vec<RistrettoPoint>>();

        PublicSet(public_keys)
    }

    // Returns all of the keyImages for a specific private key set
    // We calculate the key image using the formula keyImage = privateKey * HashToPoint(PublicSigningKey)
    // The difference here is that we compute the key images with respect to the hash of the
    // public key corresponding to the signing key
    // Note that the HashToPoint must not allow the basepoint in the public key to be factored out
    pub fn compute_key_images(
        &self,
        signers_basepoint: &RistrettoPoint,
    ) -> Vec<CompressedRistretto> {
        self.0
            .iter()
            .map(|priv_key| (priv_key * signers_basepoint).compress())
            .collect()
    }

    // Returns the number of private keys in the set
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests_helper::*;
    // This test is a sanity check for private to public key sets.
    // The iter method is used when converting from a set of private keys
    // to a set of public keys. In the test, we use a for loop and check that both
    // are equal.
    #[test]
    fn private_set_to_public_set() {
        let private_set = generate_private_set(10);
        let public_set = private_set.to_public_set();

        assert_eq!(private_set.len(), public_set.len());

        for i in 0..private_set.len() {
            match (private_set.0.get(i), public_set.0.get(i)) {
                (Some(private_key), Some(expected_public_key)) => {
                    let public_key = private_key * &BASEPOINT;
                    assert_eq!(public_key, *expected_public_key);
                }
                _ => panic!("could not get the private/public key at index {} ", i),
            }
        }
    }
    #[test]
    fn check_duplicates_exist() {
        let private_set = generate_private_set(10);
        let mut public_set = private_set.to_public_set();

        let dup_exists = public_set.duplicates_exist();
        assert!(!dup_exists);

        let last_element = public_set.0.last().unwrap().clone();
        public_set.0[0] = last_element;

        let dup_exists = public_set.duplicates_exist();
        assert!(dup_exists);
    }
}