use polars::prelude::*;
use serde::{Deserialize, Serialize};

use super::base::{DfBaseData, DfColumns};

#[derive(Deserialize, Serialize, Debug)]
pub struct BollingerBandsData {
    #[serde(flatten)]
    pub base_data: DfBaseData,
    pub ma_windows: String,
    pub upper_band: String,
    pub lower_band: String,
    pub signal: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BollingerBandsResponse {
    pub columns: DfColumns,
    pub data: Vec<BollingerBandsData>,
}

impl BollingerBandsData {
    pub fn new() -> Self {
        let base_data = DfBaseData::new();
        BollingerBandsData { 
            base_data,
            ma_windows: String::new(),
            upper_band: String::new(),
            lower_band: String::new(),
            signal: String::new()
        }
    }
}

impl BollingerBandsResponse {
    pub fn new(df_columns: DfColumns, df_data: Vec<BollingerBandsData>) -> Self {
        BollingerBandsResponse {
            columns: df_columns,
            data: df_data
        }
    }
}
pub struct BollingerBandsConverter;

impl BollingerBandsConverter {
    pub fn convert_rows(df: &DataFrame) -> Vec<BollingerBandsData> {
        let mut data_response: Vec<BollingerBandsData> = Vec::new();
        for row in 0..df.height() {
            let mut temp = BollingerBandsData::new();
            for col in df.get_columns() {
                match col.name().as_str() {
                    "high" | "low" | "open" | 
                    "close" | "datetime" => temp.base_data.set_base_data(df, col, row),
                    name if name.contains("Sig") => temp.signal = df.column(col.name())
                                                                    .unwrap()
                                                                    .i32()
                                                                    .unwrap()
                                                                    .get(row)
                                                                    .unwrap()
                                                                    .to_string(),
                    name if name.starts_with("SMA") => {
                        temp.ma_windows = Self::get_f32_col_value(df, col.name(), row)
                    }
                    name if name.starts_with("Upper") => {
                        temp.upper_band = Self::get_f32_col_value(df, col.name(), row)
                    }
                    name if name.starts_with("Lower") => {
                        temp.lower_band = Self::get_f32_col_value(df, col.name(), row);
                    }
                    _ => continue
                }
            }
            data_response.push(temp);
        }
        return data_response;
    }
    
    fn get_f32_col_value(df: &DataFrame, col_name: &str, row: usize) -> String {
        let value = df.column(col_name)
                        .unwrap()
                        .f32()
                        .unwrap()
                        .get(row)
                        .unwrap_or(0.0)
                        .to_string();
        if value != "0" {
            return value;
        } else {
            return "NaN".to_string();
        }
    }
}