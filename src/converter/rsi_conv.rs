use crate::processor::RSIData;
use polars::prelude::*;

pub struct RSIConverter;

impl RSIConverter {
    pub fn convert_rows(df: &DataFrame) -> Vec<RSIData> {
        let mut data_response: Vec<RSIData> = Vec::new();
        for row in 0..df.height() {
            let mut temp = RSIData::new();
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
                    name if name.contains("RSI") => {
                        let value = df.column(col.name())
                                        .unwrap()
                                        .f32()
                                        .unwrap()
                                        .get(row)
                                        .unwrap_or(0.0)
                                        .to_string();
                        if value != "0" {
                            temp.rsi = value
                        } else {
                            temp.rsi = "NaN".to_string();
                        }
                    }
                    _ => continue
                }
            }
            data_response.push(temp);
        }
        return  data_response;
    }
}
