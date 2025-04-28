use crate::error::ClientError;
use crate::rpc::RpcParams;
use jsonrpsee_core::{client::ClientT, JsonValue};
use jsonrpsee_http_client::HttpClient as JRPSHttpClient;
use sdk_core::types::{BlockId, BlockState, DispatchIndex, EmittedIndex, HashIndex, TransactionLocation, H256};
use serde::Deserialize;

use serde::Serialize;

#[derive(Clone, Deserialize)]
pub struct TransactionSignature {
	pub ss58_address: Option<String>,
	pub nonce: u32,
	pub app_id: u32,
	pub mortality: Option<(u64, u64)>, // None means the tx is Immortal
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusEvent {
	pub phase: ConsensusEventPhase,
	pub emitted_index: EmittedIndex,
	pub decoded: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConsensusEventPhase {
	// Finalizing the block.
	Finalization,
	/// Initializing the block.
	Initialization,
}

pub mod block_overview {
	use super::*;

	pub async fn request(client: &JRPSHttpClient, parameters: Params) -> Result<(Response, u64), ClientError> {
		let mut params = RpcParams::new();
		params.push(parameters)?;

		let raw_value = client
			.request::<JsonValue, _>("block_overview", params)
			.await
			.map_err(ClientError::Jsonrpsee)?;
		dbg!(&raw_value);
		let debug_execution_time = raw_value.get("debug_execution_time");
		let response = raw_value.get("value").unwrap();

		let debug_execution_time = debug_execution_time.and_then(|x| x.as_u64()).unwrap();
		let response: Response = serde_json::from_value(response.clone()).unwrap();

		Ok((response, debug_execution_time))
	}

	#[derive(Debug, Clone, Serialize)]
	pub struct Params {
		pub block_id: HashIndex,
		#[serde(default)]
		pub extension: ParamsExtension,
		#[serde(default)]
		pub filter: Filter,
	}

	#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
	pub struct ParamsExtension {
		#[serde(default)]
		pub enable_call_decoding: bool,
		#[serde(default)]
		pub fetch_events: bool,
		#[serde(default)]
		pub enable_event_decoding: bool,
		#[serde(default)]
		pub enable_consensus_event: bool,
	}

	#[derive(Debug, Default, Clone, Serialize, Deserialize)]
	pub struct Filter {
		#[serde(default)]
		pub transaction: TransactionFilterOptions,
		#[serde(default)]
		pub signature: SignatureFilterOptions,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub enum TransactionFilterOptions {
		All,
		TxHash(Vec<H256>),
		TxIndex(Vec<u32>),
		Pallet(Vec<u8>),
		PalletCall(Vec<DispatchIndex>),
		HasEvent(Vec<EmittedIndex>),
	}

	impl Default for TransactionFilterOptions {
		fn default() -> Self {
			Self::All
		}
	}

	#[derive(Debug, Default, Clone, Serialize, Deserialize)]
	pub struct SignatureFilterOptions {
		pub ss58_address: Option<String>,
		pub app_id: Option<u32>,
		pub nonce: Option<u32>,
	}

	#[derive(Clone, Deserialize)]
	pub struct Response {
		pub block_id: BlockId,
		pub block_state: BlockState,
		pub transactions: Vec<ResponseTransaction>,
		pub consensus_events: Option<Vec<ConsensusEvent>>,
	}

	#[derive(Clone, Deserialize)]
	pub struct ResponseTransaction {
		pub location: TransactionLocation,
		pub dispatch_index: DispatchIndex,
		pub signature: Option<TransactionSignature>,
		pub decoded: Option<String>,
		pub events: Option<Vec<TransactionEvents>>,
	}

	#[derive(Clone, Deserialize)]
	pub struct TransactionEvents {
		pub index: u32,
		pub emitted_index: EmittedIndex,
		pub decoded: Option<String>,
	}
}
