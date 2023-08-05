use crate::proto::beco::WalletResponse as ProtoWalletResponse;

#[derive(Debug, PartialEq)]
pub struct WalletResponse {
    pub alias: String,
    pub public_key: String,
    pub classic_address: Option<String>,
}

impl From<&WalletResponse> for ProtoWalletResponse {
    fn from(value: &WalletResponse) -> Self {
        Self {
            alias: value.alias.clone(),
            public_key: value.public_key.clone(),
            classic_address: value.classic_address.clone(),
        }
    }
}

impl From<WalletResponse> for ProtoWalletResponse {
    fn from(value: WalletResponse) -> Self {
        Self {
            alias: value.alias.clone(),
            public_key: value.public_key.clone(),
            classic_address: value.classic_address.clone(),
        }
    }
}
