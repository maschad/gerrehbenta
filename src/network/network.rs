use anyhow::Result;
use std::sync::{mpsc::Receiver, Arc};

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
            NetworkEvent::GetENSAddressInfo { name_or_address } => {
                let res = match name_or_address {
                    NameOrAddress::Name(name) => {
                        Self::get_name_info(&self.etherscan_endpoint, &name).await
                    }
                    NameOrAddress::Address(address) => {
                        Self::get_address_info(&self.etherscan_endpoint, address).await
                    }
                };
                let mut app = self.app.lock().await;
                if app.search_state.is_searching {
                    app.pop_current_route();
                }

                // if let Ok(Some(address_info)) = res {}
                // app.set_route(Route::new(
                //     RouteId::MyPositions(if let Ok(some) = position_res {
                //         some
                //     } else {
                //         None
                //     }),
                //     ActiveBlock::MyPositions,
                // ));

                app.search_state.is_searching = false;
            }
            NetworkEvent::GetAddressPositionInfo { address, positions } => {
                //#TODO Load positions
            }
        }
    }

    async fn get_name_info(endpoint: &str, ens_id: &str) -> Result<Option<AddressInfo>> {
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

    async fn get_address_info(endpoint: &str, address: Address) -> Result<Option<AddressInfo>> {
        let provider = Provider::<Http>::try_from(endpoint)?;
        let ens_id = provider.lookup_address(address).await.ok();

        let balance = provider.get_balance(address, None).await?;

        Ok(Some(AddressInfo {
            address,
            balance,
            ens_id,
        }))
    }

    // async fn get_uniswap_v3_positions(
    //     endpoint: &str,
    //     address: Address,
    // ) -> Result<Option<Vec<Position>>> {
    //     let provider = Provider::<Http>::try_from(endpoint)?;
    //     let positions = provider.await?;

    //     Ok(Some(positions))
    // }
}

#[tokio::main]
pub async fn handle_tokio(io_rx: Receiver<NetworkEvent>, network: &mut Network) {
    while let Ok(io_event) = io_rx.recv() {
        network.handle_event(io_event).await;
    }
}
