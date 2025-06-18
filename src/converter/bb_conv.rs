use crate::processor::BollingerBandsData;
use polars::prelude::*;

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