//use crate::params::*;
use sdk_client::{
	core::crypto::{Keypair, SecretUri},
	error::ClientError,
	http::Client,
	params::{Extra, Mortality, Nonce},
	rpc,
};
use sdk_core::types::avail;
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

	create_application_key(&client, &account).await?;
	submit_data(&client, &account).await?;
	manually_set_nonce(&client, &account).await?;
	manually_set_mortality(&client, &account).await?;
	manually_set_app_id(&client, &account).await?;
	fetch_best_block_hash(&client).await?;
	fetch_finalized_block_hash(&client).await?;
	fetch_genesis_hash(&client).await?;
	fetch_runtime_version(&client).await?;
	fetch_block_header(&client).await?;
	fetch_block(&client).await?;

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
