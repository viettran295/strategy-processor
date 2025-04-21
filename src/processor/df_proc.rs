use crate::fetch::TwelveDataResponse;
use crate::processor::{CrossingMAResponse, DfData, DfColumns};

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
        let mut cols_response = DfColumns::new();
        let mut data_response: Vec<DfData> = Vec::new();
        let column_names = df.get_columns()
                            .iter()
                            .map(|col| col.name().to_string())
                            .collect();
        let column_types = df.get_columns()
                            .iter()
                            .map(|col| col.dtype().to_string())
                            .collect();
        cols_response.column_names = column_names;
        cols_response.column_types = column_types;
        for row in 0..df.height() {
            let mut temp = DfData::new();
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
            data_response.push(temp);
        }
        let response = CrossingMAResponse::new(cols_response, data_response);
        return serde_json::to_string(&response).unwrap();
    }
}