use crate::{error::ClientError, http::Client, params};
use sdk_core::crypto::{AccountId, Keypair, Signature};
use sdk_core::types::avail::Nonce;
use sdk_core::types::{build_transaction, Additional, Call, UnsignedEncodedPayload, H256};

#[derive(Clone)]
pub struct SubmittableTransaction {
	client: Client,
	call: Call,
	extra: params::Extra,
}

impl SubmittableTransaction {
	pub fn new(client: Client, call: Call, extra: params::Extra) -> Self {
		Self { client, call, extra }
	}

	pub async fn sign(&self, signer: &Keypair) -> Result<(Signature, UnsignedEncodedPayload), ClientError> {
		let account_id = signer.account_id();
		let payload = self
			.client
			.build_payload(self.call.clone(), account_id, self.extra)
			.await?;
		let encoded_payload = payload.encode();

		Ok((encoded_payload.sign(signer), encoded_payload))
	}

	pub async fn sign_and_submit(&self, signer: &Keypair) -> Result<H256, ClientError> {
		let (signature, payload) = self.sign(signer).await?;
		let account_id = signer.account_id();

		let transaction = build_transaction(&payload.extra.0, &payload.call.0, account_id, signature);
		self.client.submit_transaction(&transaction).await
	}

	pub async fn sign_and_submit_extra_info(
		&self,
		signer: &Keypair,
	) -> Result<ExtraTransactionInformation, ClientError> {
		let account_id = signer.account_id();
		let payload = self
			.client
			.build_payload(self.call.clone(), account_id, self.extra)
			.await?;
		let extra = payload.extra.clone();
		let additional = payload.additional.clone();

		let encoded_payload = payload.encode();
		let signature = encoded_payload.sign(signer);

		let transaction = build_transaction(&encoded_payload.extra.0, &encoded_payload.call.0, account_id, signature);
		let hash = self.client.submit_transaction(&transaction).await?;

		Ok(ExtraTransactionInformation {
			hash,
			account_id,
			extra,
			additional,
		})
	}
}

#[derive(Debug, Clone)]
pub struct ExtraTransactionInformation {
	pub hash: H256,
	pub account_id: AccountId,
	pub extra: sdk_core::types::Extra,
	pub additional: Additional,
}

impl ExtraTransactionInformation {
	pub fn nonce(&self) -> Nonce {
		self.extra.nonce.0
	}

	pub fn fork_hash(&self) -> H256 {
		self.additional.fork_hash
	}
}
