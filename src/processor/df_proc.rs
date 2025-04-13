use std::collections::HashMap;
use crate::fetch::StockDataPoint;

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

    pub fn to_df(&mut self, prices: Vec<f32>, datetime: &[&str]) {
        if prices.len() != datetime.len() {
            error!("Length of 2 columns must be equal");
            return
        }
        
        if let Ok(df) = DataFrame::new(vec![
            Series::new("close".into(), prices).into(),
            Series::new("datetime".into(), datetime).into(),
        ]) {
            self.df = Some(df);
            info!("Converted data to dataframe");
        } else {
            error!("Error converting dataframe")
        }
    }

    pub fn df_to_json(df: &DataFrame) -> String {
        let mut values_response = HashMap::new();
        let mut response: Vec<StockDataPoint > = Vec::new();
        for row in 0..df.height() {
            let mut temp = StockDataPoint { 
                datetime: String::new(),
                open: String::new(),
                close: String::new(),
                high: String::new(),
                low: String::new(),
            };
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
                        temp.close = value
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