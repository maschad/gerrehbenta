use crate::models::position::Position;
use anyhow::Result;
use serde_json::Value;

const UNISWAP_SUBGRAPH_URL: &str =
    "https://gateway.thegraph.com/api/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV";

pub async fn fetch_positions(owner: &str) -> Result<(Vec<Position>, Vec<(f64, f64)>)> {
    let query = format!(
        r#"{{
            positions(where: {{owner: "{}", liquidity_gt: 0}}) {{
                token0 {{
                    symbol
                    decimals
                    name
                    volumeUSD
                }}
                token1 {{
                    symbol
                    decimals
                    name
                    volumeUSD
                }}
                pool {{
                    token0Price
                    token1Price
                    volumeToken0
                    volumeToken1
                    feeGrowthGlobal0X128
                    feeGrowthGlobal1X128
                }}
                feeGrowthInside0LastX128
                feeGrowthInside1LastX128
                tickLower {{feeGrowthOutside0X128 feeGrowthOutside1X128}}
                tickUpper {{feeGrowthOutside0X128 feeGrowthOutside1X128}}
                withdrawnToken0
                withdrawnToken1
                depositedToken0
                depositedToken1
                liquidity
            }}
            tokenDayDatas(first: 24, orderBy: date, orderDirection: desc) {{
                date
                volumeUSD
            }}
        }}"#,
        owner
    );

    let client = reqwest::Client::new();
    let response = client
        .post(UNISWAP_SUBGRAPH_URL)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": query
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
    if let Some(errors) = data.get("errors") {
        log::error!("GraphQL errors: {:?}", errors);
        return Ok((Vec::new(), Vec::new()));
    }

    // Parse positions
    let positions = data["data"]["positions"].clone();
    let positions: Vec<Position> = serde_json::from_value(positions).unwrap_or_default();

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
