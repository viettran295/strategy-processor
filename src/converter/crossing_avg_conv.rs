use polars::prelude::*;
use serde::{Deserialize, Serialize};

use super::base::{DfBaseData, DfColumns};

#[derive(Deserialize, Serialize, Debug)]
pub struct CrossingMAData {
    #[serde(flatten)]
    pub base_data: DfBaseData,
    pub ma_windows: Vec<String>,
    pub signal: String
}

impl CrossingMAData {
    pub fn new() -> Self {
        let base_data = DfBaseData::new();
        CrossingMAData { 
            base_data,
            ma_windows: Vec::new(),
            signal: String::new()
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CrossingMAResponse {
    pub columns: DfColumns,
    pub data: Vec<CrossingMAData>,
}

impl CrossingMAResponse {
    pub fn new(df_columns: DfColumns, df_data: Vec<CrossingMAData>) -> Self {
        CrossingMAResponse {
            columns: df_columns,
            data: df_data
        }
    }
}

pub struct CrossingMAConverter;

impl CrossingMAConverter {
    pub fn convert_rows(df: &DataFrame) -> Vec<CrossingMAData> {
        let mut data_response: Vec<CrossingMAData> = Vec::new();
        for row in 0..df.height() {
            let mut temp = CrossingMAData::new();
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
                    name if name.contains("SMA") || name.contains("EWMA")=> {
                        let value = df.column(col.name())
                                        .unwrap()
                                        .f32()
                                        .unwrap()
                                        .get(row)
                                        .unwrap_or(0.0)
                                        .to_string();
                        if value != "0" {
                            temp.ma_windows.push(value);
                        } else {
                            temp.ma_windows.push("NaN".to_string());
                        }
                    }
                    _ => continue
                }
            }
            data_response.push(temp);
        }
        return data_response;
    }
}
