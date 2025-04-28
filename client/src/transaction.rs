use crate::error::ClientError;
use crate::http::Client;
use crate::params::Extra;
use sdk_core::crypto::{Keypair, Signature};
use sdk_core::types::{build_transaction, Call, UnsignedEncodedPayload, H256};

#[derive(Clone)]
pub struct SubmittableTransaction {
	client: Client,
	call: Call,
	extra: Extra,
}

impl SubmittableTransaction {
	pub fn new(client: Client, call: Call, extra: Extra) -> Self {
		Self { client, call, extra }
	}

	pub async fn sign(&self, signer: &Keypair) -> Result<(Signature, UnsignedEncodedPayload), ClientError> {
		let account_id = signer.account_id();
		let payload = self
			.client
			.build_payload(self.call.clone(), account_id, self.extra)
			.await?;

		Ok((payload.sign(signer), payload))
	}

	pub async fn sign_and_submit(&self, signer: &Keypair) -> Result<H256, ClientError> {
		let (signature, payload) = self.sign(signer).await?;
		let account_id = signer.account_id();

		let transaction = build_transaction(&payload.extra.0, &payload.call.0, account_id, signature);
		self.client.submit_transaction(&transaction).await
	}
}
