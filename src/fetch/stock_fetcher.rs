use std::error::Error;
use serde::Deserialize;
use crate::config::TwelDataCfg;

#[derive(Deserialize, Debug)]
pub struct StockDataPoint {
    pub close: String,
    pub datetime: String
}

#[derive(Deserialize, Debug)]
pub struct TwelveDataResponse {
    pub values: Vec<StockDataPoint>,
}

pub struct StockFetcher {
    pub config: TwelDataCfg,
}

impl StockFetcher {
    pub fn new() -> Self {
        let cfg = TwelDataCfg::new();
        return Self {
            config: cfg
        }
    }

    pub async fn fetch_prices(&self, stock: String) -> Result<Vec<StockDataPoint>, Box<dyn Error>> {
        let api_url = format!(
            "{}symbol={}&interval={}&outputsize={}&apikey={}",
            self.config.url, stock, self.config.interval, self.config.days, self.config.api_key
        );
        let response = reqwest::get(api_url).await?;
        if response.status().is_success() {
            let body = response.text().await?;
            let data: Result<TwelveDataResponse, serde_json::Error> = serde_json::from_str(&body);
            return Ok(data.unwrap().values);
        } else {
            return Err(format!("Failed to fetch data for '{}'. Status: {}", 
                                stock, response.status()).into())
        }
    }
}