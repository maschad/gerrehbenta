use crate::models::position::Position;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::env;

const UNISWAP_SUBGRAPH_URL: &str =
    "https://gateway.thegraph.com/api/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV";

pub async fn fetch_positions(owner: &str) -> Result<(Vec<Position>, Vec<(f64, f64)>)> {
    log::debug!("Fetching positions for owner: {}", owner);
    // Ensure the address has the 0x prefix and is lowercase
    let owner_address = if owner.starts_with("0x") {
        owner.to_lowercase()
    } else {
        format!("0x{}", owner.to_lowercase())
    };

    let query = format!(
        r#"{{
            positions(where: {{owner: "{}", liquidity_gt: 0}}) {{
                token0 {{
                    symbol
                    name
                    decimals
                    volumeUSD
                }}
                token1 {{
                    symbol
                    name
                    decimals
                    volumeUSD
                }}
                pool {{
                    id
                    token0Price
                    token1Price
                    poolHourData(first: 24, orderBy: periodStartUnix, orderDirection: desc) {{
                        periodStartUnix
                        token0Price
                        token1Price
                        volumeUSD
                    }}
                    poolDayData(first: 7, orderBy: date, orderDirection: desc) {{
                        date
                        token0Price
                        token1Price
                    }}
                }}
                tickLower {{feeGrowthOutside0X128 feeGrowthOutside1X128}}
                tickUpper {{feeGrowthOutside0X128 feeGrowthOutside1X128}}
                withdrawnToken0
                withdrawnToken1
                depositedToken0
                depositedToken1
                liquidity
                transaction {{
                    timestamp
                }}
            }}
            tokenDayDatas(first: 24, orderBy: date, orderDirection: desc) {{
                date
                volumeUSD
            }}
        }}"#,
        owner_address
    );

    let api_key = match env::var("SUBGRAPH_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            return Err(anyhow!("SUBGRAPH_API_KEY environment variable is not set. Please set it by running:\n\nexport SUBGRAPH_API_KEY=your_api_key_here\n\nYou can get an API key from https://thegraph.com/studio/apikeys/"));
        }
    };

    let client = reqwest::Client::new();
    log::debug!("Making request to Uniswap subgraph with query: {}", query);
    let mut response = match client
        .post(UNISWAP_SUBGRAPH_URL)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "query": query
        }))
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            log::error!("Failed to make request to subgraph: {}", e);
            return Err(e.into());
        }
    };

    let status = response.status();
    if !status.is_success() {
        log::error!("Subgraph returned error status: {}", status);
        let body = response.text().await?;
        log::error!("Error response body: {}", body);
        return Err(anyhow!("Subgraph request failed with status {}", status));
    }

    let response_body = response.text().await?;
    let data: Value = match serde_json::from_str(&response_body) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse subgraph response: {}", e);
            return Err(e.into());
        }
    };

    log::debug!("Received response from subgraph: {:?}", data);
    if let Some(errors) = data.get("errors") {
        log::error!("GraphQL errors: {:?}", errors);
        return Ok((Vec::new(), Vec::new()));
    }

    // Parse positions
    let positions = data["data"]["positions"].clone();
    let positions: Vec<Position> = match serde_json::from_value(positions) {
        Ok(positions) => positions,
        Err(e) => {
            log::error!("Failed to parse positions: {}", e);
            Vec::new()
        }
    };
    log::debug!("Parsed {} positions", positions.len());

    // Parse volume data
    let volume_data = data["data"]["tokenDayDatas"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|day| {
            let date = day["date"].as_i64().unwrap_or(0) as f64;
            let volume = day["volumeUSD"]
                .as_str()
                .unwrap_or("0")
                .parse::<f64>()
                .unwrap_or(0.0);
            (date, volume)
        })
        .collect();

    Ok((positions, volume_data))
}
