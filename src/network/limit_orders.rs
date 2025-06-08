use crate::app::App;
use crate::NetworkEvent;
use anyhow::Result;
use log;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub decimals: String,
    pub price: String,
    pub market_cap: String,
    pub volume: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LimitOrder {
    pub token: String,
    pub deadline: String,
    pub start_amount: String,
    pub end_amount: String,
    pub price_usd: Option<String>,
    pub value_usd: String,
    pub market_cap_usd: String,
    pub volume_24h: String,
}

fn get_token_symbol(address: &str) -> String {
    match address.to_lowercase().as_str() {
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48" => "USDC".to_string(),
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2" => "WETH".to_string(),
        "0xdac17f958d2ee523a2206206994597c13d831ec7" => "USDT".to_string(),
        "0x95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce" => "SHIB".to_string(),
        _ => address[..6].to_string() + "..." + &address[address.len() - 4..],
    }
}

fn format_amount(amount: &str, decimals: u32) -> String {
    let amount_num = amount.parse::<f64>().unwrap_or(0.0);
    let divisor = 10f64.powi(decimals as i32);
    format!("{:.6}", amount_num / divisor)
}

async fn get_coingecko_data(address: &str) -> Result<(f64, f64, f64)> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    // Normalize the address to lowercase for CoinGecko
    let address_lower = address.to_lowercase();

    let url = format!(
        "https://api.coingecko.com/api/v3/simple/token_price/ethereum?contract_addresses={}&vs_currencies=usd&include_market_cap=true&include_24hr_vol=true",
        address_lower
    );

    log::debug!("Making CoinGecko API request to {}", url);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "CoinGecko API returned error status: {}",
            response.status()
        ));
    }

    let data = response.json::<serde_json::Value>().await?;

    // First check if we have data for this token
    if !data
        .as_object()
        .map_or(false, |o| o.contains_key(&address_lower))
    {
        log::warn!(
            "No CoinGecko data found for token address: {}",
            address_lower
        );
        return Ok((0.0, 0.0, 0.0));
    }

    // Safely extract values with proper error handling
    let price = data
        .get(&address_lower)
        .and_then(|v| v.get("usd"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let market_cap = data
        .get(&address_lower)
        .and_then(|v| v.get("usd_market_cap"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let volume = data
        .get(&address_lower)
        .and_then(|v| v.get("usd_24h_vol"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    log::debug!(
        "CoinGecko data for {}: price={}, market_cap={}, volume={}",
        address_lower,
        price,
        market_cap,
        volume
    );

    Ok((price, market_cap, volume))
}

fn format_number(num: f64) -> String {
    if num < 0.01 {
        "< 0.01".to_string()
    } else if num < 1000.0 {
        format!("{:.2}", num)
    } else if num < 1_000_000.0 {
        format!("{:.2}K", num / 1000.0)
    } else if num < 1_000_000_000.0 {
        format!("{:.2}M", num / 1_000_000.0)
    } else if num < 1_000_000_000_000.0 {
        format!("{:.2}B", num / 1_000_000_000.0)
    } else {
        format!("{:.2}T", num / 1_000_000_000_000.0)
    }
}

pub async fn fetch_limit_orders(app: Arc<Mutex<App>>) -> Result<()> {
    log::debug!("Starting to fetch limit orders");

    // Check if we should use mock data from environment variable
    let use_mock_data = std::env::var("USE_MOCK_DATA")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if use_mock_data {
        log::info!("Using mock limit order data for testing");
        return fetch_mock_limit_orders(app).await;
    }

    let client = reqwest::Client::new();

    // Fetch token info from the Uniswap API
    // Using the v1 API endpoint for limit orders which is more stable
    let token_info_url = "https://api.uniswap.org/v1/limit-orders?orderStatus=open&chainId=1&limit=100&sortKey=createdAt&desc=true";

    log::debug!("Fetching limit orders from URL: {}", token_info_url);
    let response = match client.get(token_info_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to fetch limit orders: {:?}", e);
            return Ok(());
        }
    };

    if !response.status().is_success() {
        log::error!("API returned error status: {}", response.status());
        return Ok(());
    }

    let orders_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            log::error!("Failed to get response text: {:?}", e);
            return Ok(());
        }
    };

    log::debug!("Received orders response of length: {}", orders_text.len());

    let orders: serde_json::Value = match serde_json::from_str(&orders_text) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse orders JSON: {:?}", e);
            log::debug!(
                "First 100 chars of response: {}",
                &orders_text[..orders_text.len().min(100)]
            );
            return Ok(());
        }
    };

    let wallet_address = {
        let app = app.lock();
        app.wallet_address.clone()
    };

    let mut limit_orders = Vec::new();

    if let Some(orders_array) = orders.get("orders").and_then(|o| o.as_array()) {
        log::debug!("Processing {} orders", orders_array.len());
        for order in orders_array {
            if let Some(addr) = &wallet_address {
                let maker_addr = order.get("maker").and_then(|m| m.as_str());
                if let Some(m) = maker_addr {
                    if m.to_lowercase() != addr.to_lowercase() {
                        continue;
                    }
                }
            }
            // Handle possible missing fields with proper error checking
            let input = match order.get("input") {
                Some(input) => input,
                None => {
                    log::debug!("Order missing 'input' field: {:?}", order);
                    continue;
                }
            };

            let token_address = match input.get("token").and_then(|t| t.as_str()) {
                Some(addr) => addr,
                None => {
                    log::debug!("Missing token address in order");
                    continue;
                }
            };

            let token_symbol = get_token_symbol(token_address);

            let decimals = match token_symbol.as_str() {
                "USDC" | "USDT" => 6,
                "WETH" => 18,
                _ => 18, // Default to 18 for unknown tokens
            };

            // Use safe unwrapping for required fields
            let start_amount = match input.get("startAmount").and_then(|a| a.as_str()) {
                Some(amount) => format_amount(amount, decimals),
                None => {
                    log::debug!("Missing startAmount in order");
                    continue;
                }
            };

            let end_amount = match input.get("endAmount").and_then(|a| a.as_str()) {
                Some(amount) => format_amount(amount, decimals),
                None => {
                    log::debug!("Missing endAmount in order");
                    continue;
                }
            };

            let deadline = match order.get("createdAt").and_then(|d| d.as_str()) {
                Some(date_str) => match chrono::DateTime::parse_from_rfc3339(date_str) {
                    Ok(date) => date.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
                    Err(_) => "Unknown date".to_string(),
                },
                None => "Unknown date".to_string(),
            };

            // Get price data from CoinGecko with proper rate limiting
            log::debug!(
                "Fetching price data for {} at address {}",
                token_symbol,
                token_address
            );

            // Add some delay to avoid rate limiting (CoinGecko has limits for free API)
            tokio::time::sleep(Duration::from_millis(300)).await;

            let (price, market_cap, volume) = match get_coingecko_data(token_address).await {
                Ok((p, mc, v)) => (p, mc, v),
                Err(e) => {
                    log::warn!("CoinGecko API error for {}: {}", token_symbol, e);
                    (0.0, 0.0, 0.0)
                }
            };

            // Safely parse the amount
            let start_amount_num = start_amount.parse::<f64>().unwrap_or(0.0);
            let value_usd = start_amount_num * price;

            log::debug!(
                "Adding limit order for {} with price {}",
                token_symbol,
                price
            );
            limit_orders.push(LimitOrder {
                token: token_symbol,
                deadline,
                start_amount,
                end_amount,
                price_usd: Some(format_number(price)),
                value_usd: format_number(value_usd),
                market_cap_usd: format_number(market_cap),
                volume_24h: format_number(volume),
            });
        }
    }

    log::debug!(
        "Processed limit orders, found {} valid orders",
        limit_orders.len()
    );

    log::debug!("Updating app with {} limit orders", limit_orders.len());
    let mut app = app.lock();
    app.update_limit_orders(limit_orders);
    drop(app);

    Ok(())
}

// Function that creates mock limit order data for testing
async fn fetch_mock_limit_orders(app: Arc<Mutex<App>>) -> Result<()> {
    log::debug!("Generating mock limit order data");

    // Create mock limit orders
    let mock_orders = vec![
        LimitOrder {
            token: "WETH".to_string(),
            deadline: chrono::Utc::now().to_rfc3339(),
            start_amount: "1.5".to_string(),
            end_amount: "1.45".to_string(),
            price_usd: Some("3,200.00".to_string()),
            value_usd: "4,800.00".to_string(),
            market_cap_usd: "300.12B".to_string(),
            volume_24h: "12.5B".to_string(),
        },
        LimitOrder {
            token: "USDC".to_string(),
            deadline: chrono::Utc::now().to_rfc3339(),
            start_amount: "5000".to_string(),
            end_amount: "4990".to_string(),
            price_usd: Some("1.00".to_string()),
            value_usd: "5,000.00".to_string(),
            market_cap_usd: "42.5B".to_string(),
            volume_24h: "6.8B".to_string(),
        },
        LimitOrder {
            token: "SHIB".to_string(),
            deadline: chrono::Utc::now().to_rfc3339(),
            start_amount: "10000000".to_string(),
            end_amount: "9950000".to_string(),
            price_usd: Some("< 0.01".to_string()),
            value_usd: "250.00".to_string(),
            market_cap_usd: "6.2B".to_string(),
            volume_24h: "180.5M".to_string(),
        },
        LimitOrder {
            token: "UNI".to_string(),
            deadline: chrono::Utc::now().to_rfc3339(),
            start_amount: "250".to_string(),
            end_amount: "248".to_string(),
            price_usd: Some("8.75".to_string()),
            value_usd: "2,187.50".to_string(),
            market_cap_usd: "4.8B".to_string(),
            volume_24h: "145.2M".to_string(),
        },
        LimitOrder {
            token: "USDT".to_string(),
            deadline: chrono::Utc::now().to_rfc3339(),
            start_amount: "3500".to_string(),
            end_amount: "3485".to_string(),
            price_usd: Some("1.00".to_string()),
            value_usd: "3,500.00".to_string(),
            market_cap_usd: "95.7B".to_string(),
            volume_24h: "42.3B".to_string(),
        },
    ];

    // Add some random delay to simulate network latency
    log::debug!("Simulating network latency...");
    tokio::time::sleep(Duration::from_millis(500)).await;

    log::debug!("Updating app with {} mock limit orders", mock_orders.len());
    let mut app = app.lock();
    app.update_limit_orders(mock_orders);

    // Schedule next update for mock data
    let network_txn = app.network_txn.clone();
    drop(app);

    if let Some(tx) = network_txn {
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
            let _ = tx.send(NetworkEvent::FetchLimitOrders);
        });
    }

    Ok(())
}
