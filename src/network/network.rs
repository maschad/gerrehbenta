use anyhow::Result;
use std::sync::{mpsc::Receiver, Arc};

use super::ethers::types::AddressInfo;
use super::limit_orders::{fetch_limit_orders, LimitOrder};
use crate::app::Mode;
use crate::{
    app::App,
    models::position::Position,
    network::server::fetch_positions,
    routes::{ActiveBlock, Route, RouteId},
    widgets::chart::TokenChart,
};
use ethers::{
    core::types::{Address, NameOrAddress},
    providers::{Http, Middleware, Provider},
};
use parking_lot::{Mutex, RwLock};
use serde::Deserialize;
use std::convert::TryFrom;
use std::sync::mpsc::Sender;

#[derive(Deserialize, Clone, Debug)]
pub struct Etherscan {
    api_key: Option<String>,
}

#[derive(Debug)]
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

pub struct Network {
    uniswap_v3_endpoint: String,
    etherscan_endpoint: String,
    uniswap_limits_endpoint: String,
    app: Arc<Mutex<App>>,
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
                log::debug!("Handling GetENSAddressInfo event");
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
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!("Failed to resolve ENS or address: {}", e);
                        let mut app = self.app.lock();
                        app.search_state
                            .ens_state
                            .set_error(format!("ENS not found: {}", e));
                        app.search_state.is_searching = false;
                        app.search_state.ens_state.is_searching = false;
                        return Ok(());
                    }
                };

                // Update app state with the resolved address
                log::debug!("Found address info: {:?}", address_info);
                let mut app = self.app.lock();

                // Set up the UI to display the address information
                app.search_state.is_searching = false;
                app.wallet_address = Some(address_info.address.to_string());

                // Fetch positions for the wallet address
                let full_address = format!("{:?}", address_info.address);
                log::debug!("Fetching positions for address: {}", full_address);
                match fetch_positions(&full_address).await {
                    Ok((positions, volume_data)) => {
                        log::debug!("Successfully fetched {} positions", positions.len());
                        let positions_clone = positions.clone();
                        // Parse tokenDayDatas for each position
                        let token_day_datas: Vec<Vec<(f64, f64)>> = positions_clone
                            .iter()
                            .map(|pos| {
                                pos.pool
                                    .pool_day_datas
                                    .iter()
                                    .map(|d| {
                                        let date = d.date;
                                        let volume = d.token0Price.parse::<f64>().unwrap_or(0.0); // fallback: use token0Price as volume if no volume field
                                        (date, volume)
                                    })
                                    .collect()
                            })
                            .collect();
                        app.positions = positions;
                        app.stateful_table
                            .update_positions(&positions_clone, &token_day_datas);
                        app.mode = Mode::MyPositions;
                        app.change_active_block(ActiveBlock::MyPositions);
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("Failed to fetch positions: {}", e);
                        // If it's an API key error, propagate it up to exit the program
                        if e.to_string().contains("SUBGRAPH_API_KEY") {
                            Err(e)
                        } else {
                            // For other errors, show them in the UI
                            app.search_state.ens_state.set_error(e.to_string());
                            app.search_state.is_searching = false;
                            Ok(())
                        }
                    }
                }
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
pub async fn handle_tokio(io_rx: Receiver<NetworkEvent>, network: &mut Network) -> Result<()> {
    loop {
        match io_rx.recv() {
            Ok(io_event) => {
                if let Err(e) = network.handle_event(io_event).await {
                    log::error!("Error handling network event: {}", e);
                    return Err(e);
                }
            }
            Err(e) => {
                log::error!("Network channel error: {}", e);
                // Don't exit, just continue the loop
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}
