use std::env;
use dotenv::dotenv;
use log::error;

pub struct TwelDataCfg {
    pub api_key: String,
    pub url: String,
    pub interval: String,
    pub days: i32,
}
impl TwelDataCfg {
    pub fn new() -> Self {
        match dotenv() {
            Ok(_) => (),
            Err(_) => error!("Fail to load .env"),
        };
        return Self { 
            api_key: env::var("TWEL_DATA_KEY").expect("API key for 12 Data is not set"), 
            url: String::from("https://api.twelvedata.com/time_series?"),
            interval: String::from("1day"),
            days: 5
        }
    }
}
