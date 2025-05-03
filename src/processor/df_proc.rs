use crate::fetch::TwelveDataResponse;
use crate::processor::{CrossingMAResponse, CrossingMAData, 
                        DfColumns, RSIData, RSIResponse};

use std::fmt::Debug;

use log::{debug, error, info};
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
        let mut high: Vec<f32> = Vec::new();
        let mut low: Vec<f32> = Vec::new();
        let mut open: Vec<f32> = Vec::new();
        let mut close: Vec<f32> = Vec::new();
        for data_point in data.values {
            datetime.push(data_point.datetime);
            match data_point.high.parse::<f32>() {
                Ok(float_val) => high.push(float_val),
                Err(e) => {
                    error!("Error converting price values: {}", e);
                }
            }
            match data_point.low.parse::<f32>() {
                Ok(float_val) => low.push(float_val),
                Err(e) => {
                    error!("Error converting price values: {}", e);
                }
            }
            match data_point.open.parse::<f32>() {
                Ok(float_val) => open.push(float_val),
                Err(e) => {
                    error!("Error converting price values: {}", e);
                }
            }
            match data_point.close.parse::<f32>() {
                Ok(float_val) => close.push(float_val),
                Err(e) => {
                    error!("Error converting price values: {}", e);
                }
            }
        }

        if let Ok(df) = DataFrame::new(vec![
            Series::new("datetime".into(), datetime).into(),
            Series::new("high".into(), high).into(),
            Series::new("low".into(), low).into(),
            Series::new("open".into(), open).into(),
            Series::new("close".into(), close).into(),
        ]) {
            self.df = Some(df);
            info!("Converted data to dataframe");
            return Ok(());
        } else {
            Err("Error converting dataframe".into())
        }
    }

    fn get_cols_info(df: &DataFrame) -> DfColumns {
        let mut cols_response = DfColumns::new();
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
        return cols_response;
    }
    
    pub fn df_to_json(df: &DataFrame) -> String {
        let cols_response = Self::get_cols_info(&df);
        let mut data_response: Vec<CrossingMAData> = Vec::new();
        for row in 0..df.height() {
            let mut temp = CrossingMAData::new();
            for col in df.get_columns() {
                match col.dtype() {
                    DataType::Float32 => {
                        let value = df.column(col.name())
                                        .unwrap()
                                        .f32()
                                        .unwrap()
                                        .get(row)
                                        .unwrap_or(0.0)
                                        .to_string();
                        match col.name().as_str() {
                            "high" => temp.base_data.high = value,
                            "low" => temp.base_data.low = value,
                            "open" => temp.base_data.open = value,
                            "close" => temp.base_data.close = value,
                            name if name.contains("short") && value == "0" => temp.short_ma = "NaN".to_string(),
                            name if name.contains("long") && value == "0" => temp.long_ma = "NaN".to_string(),
                            name if name.contains("short") && value != "0" => temp.short_ma = value,
                            name if name.contains("long") && value != "0" => temp.long_ma = value,
                            _ => debug!("No matching column name")
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
                        temp.base_data.datetime = value
                    }
                    _ => continue
                }
            }
            data_response.push(temp);
        }
        let response = CrossingMAResponse::new(cols_response, data_response);
        return serde_json::to_string(&response).unwrap();
    }
    
    pub fn RSI_df_to_json(df: &DataFrame) -> String {
        let cols_response = Self::get_cols_info(&df);
        let mut data_response: Vec<RSIData> = Vec::new();
        for row in 0..df.height() {
            let mut temp = RSIData::new();
            for col in df.get_columns() {
                match col.dtype() {
                    DataType::Float32 => {
                        let value = df.column(col.name())
                                        .unwrap()
                                        .f32()
                                        .unwrap()
                                        .get(row)
                                        .unwrap_or(0.0)
                                        .to_string();
                        match col.name().as_str() {
                            "high" => temp.base_data.high = value,
                            "low" => temp.base_data.low = value,
                            "open" => temp.base_data.open = value,
                            "close" => temp.base_data.close = value,
                            name if name.contains("RSI") && value == "0" => temp.rsi = "NaN".to_string(),
                            name if name.contains("RSI") && value != "0" => temp.rsi = value,
                            _ => debug!("No matching column name")
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
                        temp.base_data.datetime = value
                    }
                    _ => continue
                }
            }
            data_response.push(temp);
        }
        let response = RSIResponse::new(cols_response, data_response);
        return serde_json::to_string(&response).unwrap();
    }
}