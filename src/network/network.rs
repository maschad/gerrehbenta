use std::{error::Error, sync::Arc};

use super::ethers::types::AddressInfo;
use crate::{
    app::{self, App},
    models::position::Position,
};
use ethers::{
    core::types::{Address, NameOrAddress},
    etherscan::Client,
    providers::{Http, Middleware, Provider},
    types::Chain,
};
use futures::future::try_join;
use serde::Deserialize;
use std::convert::TryFrom;
use tokio::sync::Mutex;

#[derive(Deserialize, Clone, Debug)]
pub struct Etherscan {
    api_key: Option<String>,
}

#[derive(Clone)]
pub struct Network {
    endpoint: String,
    app: Arc<Mutex<App>>,
}

pub enum NetworkEvent {
    GetENSAddressInfo {
        name_or_address: NameOrAddress,
        is_searching: bool,
    },
    GetAddressInfo {
        address: Address,
        positions: Option<Vec<Position>>,
    },
}

impl Network {
    pub fn default(app: Arc<Mutex<App>>, endpoint: String) -> Self {
        Self { endpoint, app }
    }

    pub async fn handle_event(&self, event: NetworkEvent) {}

    async fn get_name_info(
        endpoint: String,
        ens_id: String,
    ) -> Result<Option<AddressInfo>, Box<dyn Error>> {
        let provider = Provider::<Http>::try_from(endpoint)?;
        let address = provider.resolve_name(&ens_id).await?;

        let avatar_url = provider.resolve_avatar(&ens_id).await.ok();
        let balance = provider
            .get_balance(address, None /* //#TODO handle error */)
            .await?;
        //#TODO: Have an error type for this
        Ok(Some(AddressInfo {
            address,
            balance,
            ens_id: Some(ens_id.to_owned()),
        }))
    }

    async fn get_address_info(
        endpoint: String,
        address: Address,
    ) -> Result<Option<AddressInfo>, Box<dyn Error>> {
        let provider = Provider::<Http>::try_from(endpoint)?;
        let ens_id = provider.lookup_address(address).await.ok();

        let avatar_url = if let Some(ens_id) = ens_id.as_ref() {
            provider.resolve_avatar(ens_id).await.ok()
        } else {
            None
        };

        let (contract_source_code, contract_abi) =
            if let Ok(client) = Client::new_from_env(Chain::Mainnet) {
                try_join(
                    client.contract_source_code(address),
                    client.contract_abi(address),
                )
                .await
                .map_or((None, None), |res| (Some(res.0), Some(res.1)))
            } else {
                (None, None)
            };

        let balance = provider.get_balance(address, None).await?;

        Ok(Some(AddressInfo {
            address,
            balance,
            ens_id,
        }))
    }
}
