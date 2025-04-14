use std::error::Error;
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

use crate::config::TwelDataCfg;

#[derive(Deserialize, Serialize, Debug)]
pub struct StockDataPoint {
    pub datetime: String,
    pub open: String,
    pub close: String,
    pub high: String,
    pub low: String
}

#[derive(Deserialize, Debug)]
pub struct TwelveDataResponse {
    pub values: Vec<StockDataPoint>,
}

pub struct StockFetcher {
    pub config: TwelDataCfg,
    pub start_date: String,
    pub end_date: String,
}

impl StockFetcher {
    pub fn new() -> Self {
        let cfg = TwelDataCfg::new();
        let end_date = Utc::now().date_naive();
        // By default is 3 years data
        let start_date = end_date - Duration::days(3 * 365);
        return Self {
            config: cfg,
            start_date: start_date.to_string(),
            end_date: end_date.to_string()
        }
    }

    pub async fn fetch_prices(
        &self, 
        stock: String, 
        start_date: Option<String>, 
        end_date: Option<String>
    ) -> Result<String, Box<dyn Error>> {
        let start_date = start_date.unwrap_or(self.start_date.clone());
        let end_date = end_date.unwrap_or(self.end_date.clone());
        let api_url = format!(
            "{}symbol={}&interval={}&start_date={}&end_date={}&apikey={}",
            self.config.url, stock, self.config.interval, start_date, end_date, self.config.api_key
        );
        
        let response = reqwest::get(api_url).await?;
        if response.status().is_success() {
            let body = response.text().await?;
            return Ok(body);
        } else {
            return Err(format!("Failed to fetch data for '{}'. Status: {}", 
                                stock, response.status()).into());
        }
    }
}