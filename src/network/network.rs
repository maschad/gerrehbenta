use std::{
    error::Error,
    sync::{mpsc::Receiver, Arc},
};

use super::ethers::types::AddressInfo;
use crate::{
    app::App,
    models::position::Position,
    routes::{ActiveBlock, Route, RouteId},
};
use ethers::{
    core::types::{Address, NameOrAddress},
    providers::{Http, Middleware, Provider},
};
use serde::Deserialize;
use std::convert::TryFrom;
use tokio::sync::Mutex;

#[derive(Deserialize, Clone, Debug)]
pub struct Etherscan {
    api_key: Option<String>,
}

#[derive(Clone)]
pub struct Network {
    uniswap_v3_endpoint: String,
    etherscan_endpoint: String,
    app: Arc<Mutex<App>>,
}

pub enum NetworkEvent {
    GetENSAddressInfo {
        name_or_address: NameOrAddress,
        is_searching: bool,
    },
    GetAddressPositionInfo {
        address: Address,
        positions: Option<Vec<Position>>,
    },
}

impl Network {
    pub fn default(
        app: Arc<Mutex<App>>,
        etherscan_endpoint: String,
        uniswap_v3_endpoint: String,
    ) -> Self {
        Self {
            etherscan_endpoint,
            uniswap_v3_endpoint,
            app,
        }
    }

    pub async fn handle_event(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::GetENSAddressInfo {
                name_or_address,
                is_searching,
            } => {
                let res = match name_or_address {
                    NameOrAddress::Name(name) => {
                        Self::get_name_info(&self.etherscan_endpoint, &name).await
                    }
                    NameOrAddress::Address(address) => {
                        Self::get_address_info(&self.etherscan_endpoint, address).await
                    }
                };
                let mut app = self.app.lock().await;
                if is_searching {
                    app.pop_current_route();
                }
                app.set_route(Route::new(
                    RouteId::AddressInfo(if let Ok(some) = res { some } else { None }),
                    ActiveBlock::MyPositions,
                ));

                app.is_loading = false;
            }
            NetworkEvent::GetAddressPositionInfo { address, positions } => {
                //#TODO Load positions
            }
        }
    }

    async fn get_name_info(
        endpoint: &str,
        ens_id: &str,
    ) -> Result<Option<AddressInfo>, Box<dyn Error>> {
        let provider = Provider::<Http>::try_from(endpoint)?;
        let address = provider.resolve_name(&ens_id).await?;

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
        endpoint: &str,
        address: Address,
    ) -> Result<Option<AddressInfo>, Box<dyn Error>> {
        let provider = Provider::<Http>::try_from(endpoint)?;
        let ens_id = provider.lookup_address(address).await.ok();

        let balance = provider.get_balance(address, None).await?;

        Ok(Some(AddressInfo {
            address,
            balance,
            ens_id,
        }))
    }
}

#[tokio::main]
pub async fn handle_tokio(io_rx: Receiver<NetworkEvent>, network: &mut Network) {
    while let Ok(io_event) = io_rx.recv() {
        network.handle_event(io_event).await;
    }
}
