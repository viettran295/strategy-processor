use crate::processor::CrossingMAData;
use polars::prelude::*;

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
                    name if name.contains("short") => {
                        let value = df.column(col.name())
                                        .unwrap()
                                        .f32()
                                        .unwrap()
                                        .get(row)
                                        .unwrap_or(0.0)
                                        .to_string();
                        if value != "0" {
                            temp.short_ma = value;
                        } else {
                            temp.short_ma = "NaN".to_string();
                        }
                    }
                    name if name.contains("long") => {
                        let value = df.column(col.name())
                                        .unwrap()
                                        .f32()
                                        .unwrap()
                                        .get(row)
                                        .unwrap_or(0.0)
                                        .to_string();
                        if value != "0" {
                            temp.long_ma = value;
                        } else {
                            temp.long_ma= "NaN".to_string();
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
