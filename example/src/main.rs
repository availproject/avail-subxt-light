//use crate::params::*;
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

	println!("Create Application Key Example");
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
	fetch_kate_query_rows(&client, &account).await?;

	Ok(())
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
	let header = rpc::fetch_block(&client.client, None).await?;
	println!("{:?}", header);

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
