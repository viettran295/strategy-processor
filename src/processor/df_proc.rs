use std::collections::HashMap;
use crate::fetch::TwelveDataResponse;
use crate::processor::CrossingMAResponse;

use log::{error, info};
use polars::prelude::*;

#[derive(Clone, Debug)]
pub struct DfProcessor {
    pub df: Option<DataFrame>,
}

impl DfProcessor {
    pub fn new() -> Self {
        return Self {
            df: None
        }
    }

    pub fn to_df(&mut self, twelve_data_resp: String) -> Result<(), Box<dyn std::error::Error>> {
        let data: TwelveDataResponse = serde_json::from_str(twelve_data_resp.as_str())?;
        let mut  datetime: Vec<String> = Vec::new();
        let mut prices: Vec<f32> = Vec::new();
        for data_point in data.values {
            datetime.push(data_point.datetime);
            match data_point.close.parse::<f32>() {
                Ok(float_val) => prices.push(float_val),
                Err(e) => {
                    error!("Error converting price values: {}", e);
                }
            }
        }
        if prices.len() != datetime.len() {
            return Err("Length of 2 columns must be equal".into());
        }
        
        if let Ok(df) = DataFrame::new(vec![
            Series::new("datetime".into(), datetime).into(),
            Series::new("close".into(), prices).into(),
        ]) {
            self.df = Some(df);
            info!("Converted data to dataframe");
            return Ok(());
        } else {
            Err("Error converting dataframe".into())
        }
    }

    pub fn df_to_json(df: &DataFrame) -> String {
        let mut values_response = HashMap::new();
        let mut response: Vec<CrossingMAResponse> = Vec::new();
        for row in 0..df.height() {
            let mut temp = CrossingMAResponse::new();
            for col in df.get_columns() {
                match col.dtype() {
                    DataType::Float32 => {
                        let value = df.column(col.name())
                                        .unwrap()
                                        .f32()
                                        .unwrap()
                                        .get(row)
                                        .unwrap()
                                        .to_string();
                        if col.name().contains("close") {
                                temp.close = value
                        } else if col.name().contains("short") {
                            temp.short_ma = value;
                        } else if col.name().contains("long") {
                            temp.long_ma = value;
                        }
                    }
                    DataType::Int32 => {
                        let value = df.column(col.name())
                                                .unwrap()
                                                .i32()
                                                .unwrap()
                                                .get(row)
                                                .unwrap()
                                                .to_string();
                        temp.signal = value
                    }
                    DataType::String => {
                        let value = df.column(col.name())
                                                .unwrap()
                                                .str()
                                                .unwrap()
                                                .get(row)
                                                .unwrap()
                                                .to_string();
                        temp.datetime = value
                    }
                    _ => continue
                }
            }
            response.push(temp);
        }
        values_response.insert("values", response);
        return serde_json::to_string(&values_response).unwrap();
    }
}