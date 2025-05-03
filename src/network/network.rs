use anyhow::Result;
use std::sync::{mpsc::Receiver, Arc};

use super::ethers::types::AddressInfo;
use super::limit_orders::{fetch_limit_orders, LimitOrder};
use crate::{
    app::App,
    models::position::Position,
    routes::{ActiveBlock, Route, RouteId},
};
use ethers::{
    core::types::{Address, NameOrAddress},
    providers::{Http, Middleware, Provider},
};
use parking_lot::{Mutex, RwLock};
use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Deserialize, Clone, Debug)]
pub struct Etherscan {
    api_key: Option<String>,
}

#[derive(Clone)]
pub struct Network {
    uniswap_v3_endpoint: String,
    etherscan_endpoint: String,
    uniswap_limits_endpoint: String,
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
    FetchLimitOrders,
}

impl Network {
    pub fn default(
        app: Arc<Mutex<App>>,
        etherscan_endpoint: String,
        uniswap_v3_endpoint: String,
        uniswap_limits_endpoint: String,
    ) -> Self {
        Self {
            etherscan_endpoint,
            uniswap_v3_endpoint,
            uniswap_limits_endpoint,
            app,
        }
    }

    pub async fn handle_event(&mut self, event: NetworkEvent) -> Result<()> {
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
                // Handle the result of the name or address lookup
                let address_info = match res {
                    Ok(Some(info)) => info,
                    Ok(None) => {
                        log::warn!("No address info found for the query");
                        let mut app = self.app.lock();
                        app.search_state.is_searching = false;
                        app.messages.push("No address found for the query".to_string());
                        return Ok(());
                    },
                    Err(e) => {
                        log::error!("Failed to resolve ENS or address: {}", e);
                        let mut app = self.app.lock();
                        app.search_state.is_searching = false;
                        app.messages.push(format!("Error: {}", e));
                        return Ok(());
                    }
                };
                
                // Update app state with the resolved address
                log::debug!("Found address info: {:?}", address_info);
                let mut app = self.app.lock();
                
                if app.search_state.is_searching {
                    app.pop_current_route();
                }
                
                // Set up the UI to display the address information
                app.search_state.is_searching = false;
                app.messages.push(format!(
                    "Found address: {} (Balance: {} ETH){}",
                    address_info.address,
                    ethers::utils::format_ether(address_info.balance),
                    address_info.ens_id.map_or("".to_string(), |ens| format!(" ENS: {}", ens))
                ));
                
                // Trigger a fetch of limit orders for this address
                if let Some(tx) = &app.network_txn {
                    let _ = tx.send(NetworkEvent::FetchLimitOrders);
                }
                
                // Switch to the LimitOrders mode to show the results
                app.mode = crate::app::Mode::LimitOrders;
                app.change_active_block(ActiveBlock::LimitOrders);
                
                Ok(())
            }
            NetworkEvent::GetAddressPositionInfo { address, positions } => {
                //#TODO Load positions
                Ok(())
            }
            NetworkEvent::FetchLimitOrders => {
                let app = self.app.clone();
                fetch_limit_orders(app).await?;

                // Schedule next update
                let network_txn = self.app.lock().network_txn.clone();
                if let Some(tx) = network_txn {
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                        let _ = tx.send(NetworkEvent::FetchLimitOrders);
                    });
                }

                Ok(())
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
    loop {
        match io_rx.recv() {
            Ok(io_event) => {
                if let Err(e) = network.handle_event(io_event).await {
                    eprintln!("Error handling network event: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Network channel error: {}", e);
                // Don't exit, just continue the loop
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}
