use sdk_core::types::error::CoreError;

#[derive(Debug)]
pub enum ClientError {
	Jsonrpsee(jsonrpsee_core::client::error::Error),
	Core(CoreError),
	CodecError(parity_scale_codec::Error),
	SerdeJson(serde_json::Error),
	FromHexError(hex::FromHexError),
}
impl From<CoreError> for ClientError {
	fn from(value: CoreError) -> Self {
		ClientError::Core(value)
	}
}
impl From<parity_scale_codec::Error> for ClientError {
	fn from(value: parity_scale_codec::Error) -> Self {
		ClientError::CodecError(value)
	}
}
impl From<jsonrpsee_core::client::error::Error> for ClientError {
	fn from(value: jsonrpsee_core::client::error::Error) -> Self {
		ClientError::Jsonrpsee(value)
	}
}
impl From<hex::FromHexError> for ClientError {
	fn from(value: hex::FromHexError) -> Self {
		ClientError::FromHexError(value)
	}
}
