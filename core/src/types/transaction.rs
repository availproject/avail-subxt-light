use super::{
	multi::{MultiAddress, MultiSignature},
	H256,
};
use crate::crypto::{blake2_256, AccountId, Signature};
use parity_scale_codec::{Compact, Encode};

pub fn build_transaction(
	encoded_payload_extra: &[u8],
	encoded_payload_call: &[u8],
	account_id: AccountId,
	signature: Signature,
) -> Vec<u8> {
	let mut encoded_inner: Vec<u8> = Vec::new();

	// "is signed" + transaction protocol version (4)
	(0b10000000 + 4u8).encode_to(&mut encoded_inner);

	// Attach Address from Signer
	MultiAddress::Id(account_id).encode_to(&mut encoded_inner);

	// Attach Signature
	MultiSignature::Sr25519(signature.0).encode_to(&mut encoded_inner);

	// Attach Extra
	encoded_inner.extend(encoded_payload_extra);

	// Attach Data
	encoded_inner.extend(encoded_payload_call);

	// now, prefix byte length:
	let len = Compact(u32::try_from(encoded_inner.len()).expect("extrinsic size expected to be <4GB"));
	let mut encoded = Vec::new();
	len.encode_to(&mut encoded);
	encoded.extend(encoded_inner);

	encoded
}

pub fn hash_transaction(tx: &[u8]) -> H256 {
	H256(blake2_256(tx))
}
