//use crate::params::*;
use frame_decode::extrinsics::decode_extrinsic_current;
use frame_decode::storage::decode_storage_value_current;
use frame_metadata::RuntimeMetadata;
use frame_metadata::RuntimeMetadataPrefixed;
use parity_scale_codec::Decode;
use parity_scale_codec::Encode;
use scale_value::scale::ValueVisitor;
use sdk_core::types::avail::block::decode_raw_events;
use serde::{Deserialize, Serialize};

use scale_value::Value;
use scale_value::ValueDef;
use sdk_client::{
	core::crypto::{Keypair, SecretUri},
	core::types::{
		avail::{self, kate::Cell},
		H256,
	},
	error::ClientError,
	http::Client,
	params::{Extra, Mortality, Nonce},
	rpc,
};
use std::str::FromStr;

const METADATA_BYTES: &[u8] = include_bytes!("./../metadata.scale");

pub fn main() {
	let rt = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap();

	rt.block_on(async move { run_examples().await.expect("Cannot Fail") });
}

async fn run_examples() -> Result<(), ClientError> {
	let client = Client::new("http://127.0.0.1:9944").await?;
	let secret_uri = SecretUri::from_str("//Alice").unwrap();
	let account = Keypair::from_uri(&secret_uri).unwrap();

	let som = rpc::state_get_metadata(&client.client).await?;

	let abc = RuntimeMetadataPrefixed::decode(&mut &*som).unwrap();
	let metadata = abc.1;

	let signedBlock = rpc::fetch_block(&client.client, None).await?;
	let extrinsic_info =
		avail::block::decode_raw_extrinsics(&signedBlock.block.extrinsics, &metadata).unwrap();
	//dbg!(extrinsic_info);

	let hex_events = rpc::fetch_events(&client.client, None).await?;
	let scale_events = decode_raw_events(&hex_events, &metadata).unwrap();

	/// A phase of a block's execution.
	#[derive(Clone, Copy, PartialEq, Debug)]
	pub enum Phase {
		/// Applying an extrinsic.
		ApplyExtrinsic(u32),
		/// Finalizing the block.
		Finalization,
		/// Initializing the block.
		Initialization,
	}

	impl Phase {
		pub fn is_tx_index(&self, value: u32) -> bool {
			Self::ApplyExtrinsic(value) == *self
		}
	}

	impl TryFrom<&Value<u32>> for Phase {
		type Error = String;

		fn try_from(value: &Value<u32>) -> Result<Self, Self::Error> {
			let ValueDef::Variant(variant) = &value.value else {
				panic!()
			};

			match variant.name.as_str() {
				"ApplyExtrinsic" => (),
				"Finalization" => return Ok(Phase::Finalization),
				"Initialization" => return Ok(Phase::Initialization),
				_ => panic!(),
			}

			let Some(apply_extrinsic_value) =
				variant.values.values().next().and_then(|f| f.as_u128())
			else {
				panic!()
			};

			Ok(Self::ApplyExtrinsic(apply_extrinsic_value as u32))
		}
	}

	#[derive(Clone, PartialEq, Debug)]
	pub struct Topics {
		value: Value<u32>,
	}

	#[derive(Clone, PartialEq, Debug)]
	pub struct EventRecord {
		phase: Phase,
		event: Value<u32>,
		topics: Topics,
	}

	#[derive(Clone, PartialEq, Debug)]
	pub struct EventRecords(pub Vec<EventRecord>);

	impl TryFrom<&Vec<Value<u32>>> for EventRecords {
		type Error = String;

		fn try_from(value: &Vec<Value<u32>>) -> Result<Self, Self::Error> {
			let events: Result<Vec<EventRecord>, _> =
				value.iter().map(EventRecord::try_from).collect();
			Ok(Self(events?))
		}
	}

	impl EventRecords {
		fn find_first<T: Eventable>(&self, tx_index: Option<u32>) -> Option<Event<T>> {
			for event_record in &self.0 {
				if let Some(tx_index) = tx_index {
					if Phase::ApplyExtrinsic(tx_index) != event_record.phase {
						continue;
					}
				}

				if let Some(decoded) = Event::from_raw_event(&event_record) {
					return Some(decoded);
				}
			}

			None
		}

		fn find_last<T: Eventable>(&self, tx_index: Option<u32>) -> Option<Event<T>> {
			for event_record in self.0.iter().rev() {
				if let Some(tx_index) = tx_index {
					if Phase::ApplyExtrinsic(tx_index) != event_record.phase {
						continue;
					}
				}

				if let Some(decoded) = Event::from_raw_event(&event_record) {
					return Some(decoded);
				}
			}

			None
		}

		fn find_all<T: Eventable>(&self, tx_index: Option<u32>) -> Vec<Event<T>> {
			let mut all_events = Vec::new();
			for event_record in &self.0 {
				if let Some(tx_index) = tx_index {
					if Phase::ApplyExtrinsic(tx_index) != event_record.phase {
						continue;
					}
				}

				if let Some(decoded) = Event::from_raw_event(&event_record) {
					all_events.push(decoded);
				}
			}

			all_events
		}

		fn hash<T: Eventable>(&self, tx_index: Option<u32>) -> bool {
			for event_record in &self.0 {
				if let Some(tx_index) = tx_index {
					if Phase::ApplyExtrinsic(tx_index) != event_record.phase {
						continue;
					}
				}

				if Event::<T>::from_raw_event(&event_record).is_some() {
					return true;
				}
			}

			false
		}

		fn len(&self, tx_index: Option<u32>) -> usize {
			if let Some(tx_index) = tx_index {
				self.0
					.iter()
					.filter(|er| er.phase.is_tx_index(tx_index))
					.count()
			} else {
				self.0.len()
			}
		}

		fn data(&self) -> &Vec<EventRecord> {
			&self.0
		}
	}

	impl TryFrom<&Value<u32>> for EventRecord {
		type Error = String;

		fn try_from(value: &Value<u32>) -> Result<Self, Self::Error> {
			let scale_event = match &value.value {
				ValueDef::Composite(composite) => match composite {
					scale_value::Composite::Named(vec) => vec,
					_ => panic!(),
				},
				_ => panic!(),
			};

			let mut phase: Option<Phase> = None;
			let mut event: Option<Value<u32>> = None;
			let mut topics: Option<Topics> = None;

			for el in scale_event {
				if el.0.eq("phase") {
					phase = Some(Phase::try_from(&el.1).unwrap())
				}

				if el.0.eq("event") {
					event = Some(el.1.clone())
				}

				if el.0.eq("topics") {
					topics = Some(Topics {
						value: el.1.clone(),
					});
				}
			}

			let Some(phase) = phase else { panic!() };
			let Some(event) = event else { panic!() };
			let Some(topics) = topics else { panic!() };

			Ok(Self {
				phase,
				event,
				topics,
			})
		}
	}

	#[derive(Clone, PartialEq, Debug)]
	pub struct Event<T: Eventable> {
		phase: Phase,
		event: T,
		topics: Topics,
	}

	impl<T: Eventable> Event<T> {
		pub fn from_raw_event(value: &EventRecord) -> Option<Self> {
			let Some(decoded) = T::decode(&value.event) else {
				return None;
			};

			Some(Self {
				phase: value.phase,
				event: decoded,
				topics: value.topics.clone(),
			})
		}
	}

	let scaled_events = match scale_events.value {
		ValueDef::Composite(composite) => match composite {
			scale_value::Composite::Unnamed(vec) => vec,
			_ => panic!(),
		},
		_ => panic!(),
	};

	let a = EventRecords::try_from(&scaled_events).unwrap();
	let event = &a.data()[0].event;

	let a = a.find_first::<system::ExtrinsicSuccess>(None);
	dbg!(a);

	/* 	for ev in a.data() {
		println!("{}", get_pallet_name(&ev.event));
		println!("{}", get_event_name(&ev.event));
		println!("{:?}", get_event_data(&ev.event));
	} */

	/* 	let sv: serde_json::value::Value =
	scale_value::serde::from_value(a.data()[0].event.clone()).unwrap(); */

	/* 	println!("Create Application Key Example");
	create_application_key(&client, &account).await?;
	println!("Submit Data Example");
	submit_data(&client, &account).await?;
	println!("Manually Set Nonce Example");
	manually_set_nonce(&client, &account).await?;
	println!("Manually Set Mortality Example");
	manually_set_mortality(&client, &account).await?;
	println!("Manually Set App Id Example");
	manually_set_app_id(&client, &account).await?;
	println!("Fetch Best Block Hash Example");
	fetch_best_block_hash(&client).await?;
	println!("Fetch Finalized Block Hash Example");
	fetch_finalized_block_hash(&client).await?;
	println!("Fetch Genesis Hash Example");
	fetch_genesis_hash(&client).await?;
	println!("Fetch Runtime Version Example");
	fetch_runtime_version(&client).await?;
	println!("Fetch Block Header Example");
	fetch_block_header(&client).await?;
	println!("Fetch Block Example");
	fetch_block(&client).await?;
	println!("Fetch Kate Block Length Example");
	fetch_kate_block_length(&client).await?;
	println!("Fetch Kate Query Data Proof Example");
	fetch_kate_query_data_proof(&client, &account).await?;
	println!("Fetch Kate Query Proof Example");
	fetch_kate_query_proof(&client, &account).await?;
	println!("Fetch Kate Query Rows Example");
	fetch_kate_query_rows(&client, &account).await?; */

	Ok(())
}

pub trait Eventable: Sized {
	const PALLET_NAME: &str;
	const EVENT_NAME: &str;
	fn get_event_data(value: &Value<u32>) -> Option<&Vec<(String, Value<u32>)>> {
		if Self::PALLET_NAME.ne(&get_pallet_name(value)) {
			return None;
		}

		if Self::EVENT_NAME.ne(&get_event_name(value)) {
			return None;
		}

		Some(get_event_data(value))
	}
	fn decode(value: &Value<u32>) -> Option<Self>;
}

fn get_pallet_name(event: &Value<u32>) -> String {
	let ValueDef::Variant(variant) = &event.value else {
		panic!()
	};

	variant.name.clone()
}

fn get_event_name(event: &Value<u32>) -> String {
	let ValueDef::Variant(variant) = &event.value else {
		panic!()
	};

	let scale_value::Composite::Unnamed(vec) = &variant.values else {
		panic!()
	};

	let ValueDef::Variant(variant) = &vec[0].value else {
		panic!()
	};

	variant.name.clone()
}

fn get_event_data(event: &Value<u32>) -> &Vec<(String, Value<u32>)> {
	let ValueDef::Variant(variant) = &event.value else {
		panic!()
	};

	let scale_value::Composite::Unnamed(vec) = &variant.values else {
		panic!()
	};

	let ValueDef::Variant(variant) = &vec[0].value else {
		panic!()
	};

	let scale_value::Composite::Named(vec) = &variant.values else {
		panic!()
	};

	vec
}

pub mod system {
	use super::Eventable;
	use scale_value::Value;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
	pub struct ExtrinsicSuccess;

	impl Eventable for ExtrinsicSuccess {
		const PALLET_NAME: &str = "System";

		const EVENT_NAME: &str = "ExtrinsicSuccess";

		fn decode(value: &Value<u32>) -> Option<Self> {
			let scale_event_data = Self::get_event_data(value)?;
			dbg!(scale_event_data);

			Some(Self)
		}
	}
}

async fn create_application_key(client: &Client, account: &Keypair) -> Result<(), ClientError> {
	let account_id = account.account_id();

	let key = String::from("This is my key").as_bytes().to_vec();
	let call = avail::calls::data_availability::create_application_key(key);
	let extra = Extra::new();

	let unsigned_payload = client.build_payload(call, account_id, extra).await?;
	let signature = unsigned_payload.sign(&account);
	let transaction = client.build_transaction(&unsigned_payload, account_id, signature);

	let transaction_hash = client.submit_transaction(transaction).await?;
	println!("Transaction Hash: {}", transaction_hash.to_hex_string());

	Ok(())
}

async fn submit_data(client: &Client, account: &Keypair) -> Result<(), ClientError> {
	let account_id = account.account_id();

	let data = String::from("This is my Data").as_bytes().to_vec();
	let call = avail::calls::data_availability::submit_data(data);
	let extra = Extra::new();

	let unsigned_payload = client.build_payload(call, account_id, extra).await?;
	let signature = unsigned_payload.sign(&account);
	let transaction = client.build_transaction(&unsigned_payload, account_id, signature);

	let transaction_hash = client.submit_transaction(transaction).await?;
	println!("Transaction Hash: {}", transaction_hash.to_hex_string());

	Ok(())
}

async fn manually_set_nonce(client: &Client, account: &Keypair) -> Result<(), ClientError> {
	let account_id = account.account_id();
	let next_nonce = rpc::system_account_next_index(&client.client, &account_id).await?;

	let data = String::from("This is my Data").as_bytes().to_vec();
	let call = avail::calls::data_availability::submit_data(data);
	let extra = Extra::new().nonce(Nonce::Custom(next_nonce));

	let unsigned_payload = client.build_payload(call, account_id, extra).await?;
	let signature = unsigned_payload.sign(&account);
	let transaction = client.build_transaction(&unsigned_payload, account_id, signature);

	let transaction_hash = client.submit_transaction(transaction).await?;
	println!("Transaction Hash: {}", transaction_hash.to_hex_string());

	Ok(())
}

async fn manually_set_mortality(client: &Client, account: &Keypair) -> Result<(), ClientError> {
	let account_id = account.account_id();

	let data = String::from("This is my Data").as_bytes().to_vec();
	let call = avail::calls::data_availability::submit_data(data);
	let extra = Extra::new().mortality(Mortality::Period(8));

	let unsigned_payload = client.build_payload(call, account_id, extra).await?;
	let signature = unsigned_payload.sign(&account);
	let transaction = client.build_transaction(&unsigned_payload, account_id, signature);

	let transaction_hash = client.submit_transaction(transaction).await?;
	println!("Transaction Hash: {}", transaction_hash.to_hex_string());

	Ok(())
}

async fn manually_set_app_id(client: &Client, account: &Keypair) -> Result<(), ClientError> {
	let account_id = account.account_id();

	let data = String::from("This is my Data").as_bytes().to_vec();
	let call = avail::calls::data_availability::submit_data(data);
	let extra = Extra::new().app_id(1);

	let unsigned_payload = client.build_payload(call, account_id, extra).await?;
	let signature = unsigned_payload.sign(&account);
	let transaction = client.build_transaction(&unsigned_payload, account_id, signature);

	let transaction_hash = client.submit_transaction(transaction).await?;
	println!("Transaction Hash: {}", transaction_hash.to_hex_string());

	Ok(())
}

async fn fetch_best_block_hash(client: &Client) -> Result<(), ClientError> {
	let hash = rpc::fetch_best_block_hash(&client.client).await?;
	println!("Best Block Hash: {}", hash.to_hex_string());

	Ok(())
}

async fn fetch_finalized_block_hash(client: &Client) -> Result<(), ClientError> {
	let hash = rpc::fetch_finalized_block_hash(&client.client).await?;
	println!("Finalized Block Hash: {}", hash.to_hex_string());

	Ok(())
}

async fn fetch_genesis_hash(client: &Client) -> Result<(), ClientError> {
	let hash = rpc::chain_spec_v1_genesis_hash(&client.client).await?;
	println!("Genesis Hash: {}", hash.to_hex_string());

	Ok(())
}

async fn fetch_runtime_version(client: &Client) -> Result<(), ClientError> {
	let runtime_version = rpc::state_get_runtime_version(&client.client).await?;
	println!("{:?}", runtime_version);

	Ok(())
}

async fn fetch_block_header(client: &Client) -> Result<(), ClientError> {
	let header = rpc::fetch_block_header(&client.client, None).await?;
	println!("{:?}", header);

	Ok(())
}

async fn fetch_block(client: &Client) -> Result<(), ClientError> {
	let block = rpc::fetch_block(&client.client, None).await?;
	println!("{:?}", block);

	Ok(())
}

async fn fetch_kate_block_length(client: &Client) -> Result<(), ClientError> {
	let block_length = rpc::fetch_kate_block_length(&client.client, None).await?;
	println!("{:?}", block_length);

	Ok(())
}

async fn fetch_kate_query_data_proof(
	client: &Client,
	account: &Keypair,
) -> Result<(), ClientError> {
	wait_for_new_block(client).await?;
	_ = manually_set_app_id(client, account).await;
	let block_hash = wait_for_new_block(client).await?;
	wait_for_block_finalization(client, block_hash).await?;

	let proof_response =
		rpc::fetch_kate_query_data_proof(&client.client, 1, Some(block_hash)).await?;
	println!("{:?}", proof_response);

	Ok(())
}

async fn fetch_kate_query_proof(client: &Client, account: &Keypair) -> Result<(), ClientError> {
	wait_for_new_block(client).await?;
	_ = manually_set_app_id(client, account).await;
	let block_hash = wait_for_new_block(client).await?;
	wait_for_block_finalization(client, block_hash).await?;

	let cells = vec![Cell { row: 0, col: 0 }];
	let data_proof = rpc::fetch_kate_query_proof(&client.client, cells, Some(block_hash)).await?;
	println!("{:?}", data_proof);

	Ok(())
}

async fn fetch_kate_query_rows(client: &Client, account: &Keypair) -> Result<(), ClientError> {
	wait_for_new_block(client).await?;
	_ = manually_set_app_id(client, account).await;
	let block_hash = wait_for_new_block(client).await?;
	wait_for_block_finalization(client, block_hash).await?;

	let rows = vec![0];
	let rows = rpc::fetch_kate_query_rows(&client.client, rows, Some(block_hash)).await?;
	println!("{:?}", rows);

	Ok(())
}

async fn wait_for_new_block(client: &Client) -> Result<H256, ClientError> {
	println!("Waiting for a new block");
	let old_block_hash = rpc::fetch_best_block_hash(&client.client).await?;
	let mut new_block_hash = old_block_hash;
	while old_block_hash == new_block_hash {
		tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
		new_block_hash = rpc::fetch_best_block_hash(&client.client).await?;
	}

	Ok(new_block_hash)
}

async fn wait_for_block_finalization(client: &Client, block_hash: H256) -> Result<(), ClientError> {
	println!("Waiting for the block to finalized");
	let target_block_number = rpc::fetch_block_header(&client.client, Some(block_hash))
		.await?
		.number;

	loop {
		let finalized_block_hash = rpc::fetch_finalized_block_hash(&client.client).await?;
		let finalized_block_number =
			rpc::fetch_block_header(&client.client, Some(finalized_block_hash))
				.await?
				.number;
		if finalized_block_number >= target_block_number {
			return Ok(());
		}
		tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
	}
}
