use sdk_core::{
	crypto::AccountId,
	types::{avail, H256},
};

use crate::{error::ClientError, http::Client, rpc};

#[derive(Clone, Copy)]
pub struct Extra {
	nonce: Option<avail::Nonce>,
	mortality: Option<Mortality>,
	tip: Option<avail::Tip>,
	app_id: Option<avail::AppId>,
}
impl Extra {
	pub fn new() -> Self {
		Self {
			nonce: None,
			mortality: None,
			tip: None,
			app_id: None,
		}
	}

	pub fn nonce(mut self, value: avail::Nonce) -> Self {
		self.nonce = Some(value);
		self
	}

	pub fn mortality(mut self, value: Mortality) -> Self {
		self.mortality = Some(value);
		self
	}

	pub fn tip(mut self, value: avail::Tip) -> Self {
		self.tip = Some(value);
		self
	}

	pub fn app_id(mut self, value: avail::AppId) -> Self {
		self.app_id = Some(value);
		self
	}

	pub async fn construct(
		self,
		client: &Client,
		account_id: AccountId,
	) -> Result<(u32, Mortality, avail::Tip, avail::AppId), ClientError> {
		let tip = self.tip.unwrap_or(0u128);
		let app_id = self.app_id.unwrap_or(0);
		let nonce = match self.nonce {
			None => rpc::system_account_next_index(&client.client, account_id).await?,
			Some(n) => n,
		};
		let mortality = self.mortality.unwrap_or(Mortality::Period(32));

		Ok((nonce, mortality, tip, app_id))
	}
}

#[derive(Clone, Copy)]
pub enum Mortality {
	Period(avail::Period),
	Custom((avail::Period, avail::BlockNumber, H256)),
}
