use crate::{error::ClientError, http::Client, rpc};
use sdk_core::{
	crypto::AccountId,
	types::{avail::Nonce, BlockId, TransactionLocation, H256},
};
use std::time::Duration;
use tokio::time::sleep;

pub async fn find_block_id(
	client: &Client,
	account: (AccountId, Nonce),
	mortality: (u32, H256),
) -> Result<Option<BlockId>, ClientError> {
	let fork_height = client.block_header(Some(mortality.1)).await?.number;
	let mortality_ends_height = fork_height + mortality.0;

	let mut next_block_height = fork_height + 1;
	let mut block_height = client.finalized_block_height().await?;

	while mortality_ends_height >= next_block_height {
		if next_block_height > block_height {
			sleep(Duration::from_secs(3)).await;
			block_height = client.finalized_block_height().await?;
			continue;
		}

		let next_block_hash = client.block_hash(Some(next_block_height)).await?;
		let state_nonce = rpc::account_nonce_api_account_nonce(&client.client, &account.0, next_block_hash).await?;
		if state_nonce > account.1 {
			return Ok(Some(BlockId::new(next_block_hash, next_block_height)));
		}

		next_block_height += 1;
	}

	Ok(None)
}

pub async fn is_tx_in_block(
	client: &Client,
	tx_hash: H256,
	block_hash: H256,
) -> Result<Option<TransactionLocation>, ClientError> {
	let block = client.rpc_block(Some(block_hash)).await?;
	for (index, block_tx) in block.block.extrinsics.iter().enumerate() {
		let block_tx = hex::decode(block_tx.trim_start_matches("0x")).map_err(ClientError::FromHexError)?;
		let block_tx_hash = sdk_core::types::hash_transaction(&block_tx);
		if tx_hash == block_tx_hash {
			return Ok(Some(TransactionLocation::from((tx_hash, index as u32))));
		}
	}
	Ok(None)
}
