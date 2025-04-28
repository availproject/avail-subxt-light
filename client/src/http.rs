use super::params::{Extra, Mortality};
use crate::{error::ClientError, rpc};
use jsonrpsee_http_client::HttpClient as JRPSHttpClient;
use parity_scale_codec::Compact;
use sdk_core::{
	crypto::AccountId,
	types::{
		self,
		avail::{block::SignedBlock, BlockHeader, RuntimeVersion},
		Additional, Call, Era, UnsignedEncodedPayload, UnsignedPayload, H256,
	},
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Client {
	pub client: Arc<JRPSHttpClient>,
	genesis_hash: H256,
	runtime_version: Arc<RuntimeVersion>,
}

impl Client {
	pub async fn new(endpoint: &str) -> Result<Self, ClientError> {
		let client = JRPSHttpClient::builder().build(endpoint);
		let client = client.map_err(ClientError::Jsonrpsee)?;

		let genesis_hash = rpc::chain_spec_v1_genesis_hash(&client).await?;
		let runtime_version = Arc::new(rpc::state_get_runtime_version(&client).await?);

		Ok(Self {
			client: Arc::new(client),
			genesis_hash,
			runtime_version,
		})
	}

	pub async fn new_with_runtime(endpoint: &str, runtime: RuntimeVersion) -> Result<Self, ClientError> {
		let client = JRPSHttpClient::builder().build(endpoint);
		let client = client.map_err(ClientError::Jsonrpsee)?;

		let genesis_hash = rpc::chain_spec_v1_genesis_hash(&client).await?;
		let runtime_version = Arc::new(runtime);

		Ok(Self {
			client: Arc::new(client),
			genesis_hash,
			runtime_version,
		})
	}

	pub fn genesis_hash(&self) -> H256 {
		self.genesis_hash
	}

	pub fn runtime_version(&self) -> Arc<RuntimeVersion> {
		self.runtime_version.clone()
	}

	pub async fn fetch_best_block_hash(&self) -> Result<H256, ClientError> {
		rpc::fetch_best_block_hash(&self.client).await
	}

	pub async fn fetch_finalized_block_hash(&self) -> Result<H256, ClientError> {
		rpc::fetch_finalized_block_hash(&self.client).await
	}

	pub async fn fetch_block_header(&self, hash: Option<H256>) -> Result<BlockHeader, ClientError> {
		rpc::fetch_block_header(&self.client, hash).await
	}

	pub async fn fetch_block(&self, hash: Option<H256>) -> Result<SignedBlock, ClientError> {
		rpc::fetch_block(&self.client, hash).await
	}

	pub async fn build_payload(
		&self,
		call: Call,
		account_id: AccountId,
		extra: Extra,
	) -> Result<UnsignedEncodedPayload, ClientError> {
		let (nonce, mortality, tip, app_id) = extra.construct(self, account_id).await?;

		let app_id = Compact(app_id);
		let tip = Compact(tip);
		let nonce = Compact(nonce);
		let (mortality, fork_hash) = self.check_mortality(mortality).await?;

		let extra = types::Extra {
			mortality,
			nonce,
			tip,
			app_id,
		};

		let additional = Additional::new(
			self.runtime_version.spec_version,
			self.runtime_version.transaction_version,
			self.genesis_hash,
			fork_hash,
		);

		Ok(UnsignedPayload::new(call, extra, additional).encode())
	}

	pub async fn submit_transaction(&self, transaction: &[u8]) -> Result<H256, ClientError> {
		rpc::author_submit_extrinsic(&self.client, transaction).await
	}

	async fn check_mortality(&self, mortality: Mortality) -> Result<(Era, H256), ClientError> {
		let (era, fork_hash) = match mortality {
			Mortality::Period(period) => {
				let hash = self.fetch_finalized_block_hash().await?;
				let header = self.fetch_block_header(Some(hash)).await?;
				let number = header.number;
				(Era::mortal(period, number as u64), hash)
			},
			Mortality::Custom((period, best_number, block_hash)) => {
				(Era::mortal(period, best_number as u64), block_hash)
			},
		};

		Ok((era, fork_hash))
	}
}
