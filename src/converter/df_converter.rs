use crate::converter::bb_conv::BollingerBandsConverter;
use crate::fetch::TwelveDataResponse;
use crate::processor::{BollingerBandsData, BollingerBandsResponse, CrossingMAResponse, DfColumns, RSIResponse};
use crate::converter::CrossingMAConverter;

use std::fmt::Debug;

use log::{error, info};
use polars::prelude::*;

use super::RSIConverter;

#[derive(Clone, Debug)]
pub struct DfConverter {
    pub df: Option<DataFrame>,
}

impl DfConverter {
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

    fn get_cols_info(df: &DataFrame, exclude_col: &[&str]) -> DfColumns {
            let mut cols_response = DfColumns::new();
            cols_response.column_names = df.get_columns()
                                .iter()
                                .filter(|col| {
                                    let col_name = col.name().as_str();
                                    !exclude_col.iter().any(|&exclude| col_name.starts_with(exclude))
                                })
                                .map(|col| col.name().to_string())
                                .collect();
            cols_response.column_types = df.get_columns()
                                .iter()
                                .filter(|col| {
                                    let col_name = col.name().as_str();
                                    !exclude_col.iter().any(|&exclude| col_name.starts_with(exclude))
                                })
                                .map(|col| col.dtype().to_string())
                                .collect();
            return cols_response;
        }
    
    pub fn crossingma_df_to_json(df: &DataFrame) -> String {
        let cols_response = Self::get_cols_info(&df, &[]);
        let data_response = CrossingMAConverter::convert_rows(df);
        let response = CrossingMAResponse::new(cols_response, data_response);
        return serde_json::to_string(&response).unwrap();
    }
    
    pub fn rsi_df_to_json(df: &DataFrame) -> String {
        let exclude_cols = ["delta", "gain", "loss", "avg_gain", "avg_loss", "RS_"];
        let cols_response = Self::get_cols_info(&df, &exclude_cols);
        let data_response = RSIConverter::convert_rows(df);
        let response = RSIResponse::new(cols_response, data_response);
        return serde_json::to_string(&response).unwrap();
    }

    pub fn bb_df_to_json(df: &DataFrame) -> String {
        let exclude_cols = ["Std"];
        let cols_response = Self::get_cols_info(&df, &exclude_cols);
        let data_response = BollingerBandsConverter::convert_rows(df);
        let response = BollingerBandsResponse::new(cols_response, data_response);
        return serde_json::to_string(&response).unwrap();
    }
}