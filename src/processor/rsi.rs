use polars::prelude::*;
use log::{info, debug};
use serde::{Deserialize, Serialize};
use crate::processor::base::{DfBaseData, DfColumns};

#[derive(Deserialize, Serialize, Debug)]
pub struct RSIData {
    #[serde(flatten)]
    pub base_data: DfBaseData,
    pub rsi: String,
    pub signal: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RSIResponse {
    pub columns: DfColumns,
    pub data: Vec<RSIData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RSI {
    pub df: Option<DataFrame>,
    pub upper_bound: usize,
    pub lower_bound: usize,
    pub sma_options: RollingOptionsFixedWindow,
}

impl RSIData {
    pub fn new() -> Self {
        let base_data = DfBaseData::new();
        RSIData {
            base_data,
            rsi: String::new(),
            signal: String::new(),
        }
    }
}

impl RSIResponse {
    pub fn new(columns: DfColumns, data: Vec<RSIData>) -> Self {
        RSIResponse { columns, data }
    }
}

impl RSI {
    pub fn new(
        df: DataFrame, 
        period: usize,
        upper_bound: usize,
        lower_bound: usize
    ) -> Self {
        let sma_options = RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            weights: None,
            center: false,
            fn_params: None,
        };
        RSI {
            df: Some(df),
            sma_options,
            upper_bound,
            lower_bound
        }
    }
    pub fn calc_rsi(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            match &mut self.df {
                Some(df) => {
                    let avg_gain = format!("avg_gain_{}", self.sma_options.window_size);
                    let avg_loss = format!("avg_loss_{}", self.sma_options.window_size);
                    let rs = format!("RS_{}", self.sma_options.window_size);
                    let rsi = format!("RSI_{}", self.sma_options.window_size);

                    let mut updated_df = df.clone()
                        .lazy()
                        .with_column(
                            (col("close").shift(lit(1)) - col("close"))
                                .alias("delta")
                                .shift(lit(-1))
                        )
                        .collect()?;

                    updated_df = updated_df.clone()
                        .lazy()
                        .with_columns([
                            when(col("delta").gt(lit(0.0)))
                                .then(col("delta"))
                                .otherwise(lit(0.0))
                                .alias("gain"),
                            when(col("delta").lt(lit(0.0)))
                                .then(-col("delta"))
                                .otherwise(lit(0.0))
                                .alias("loss"),
                        ])
                        .collect()?;

                    updated_df = updated_df.clone()
                        .lazy()
                        .with_columns([
                            col("gain").rolling_mean(self.sma_options.clone())
                                        .shift(lit(-(self.sma_options.window_size as i32)))
                                        .alias(&avg_gain),
                            col("loss").rolling_mean(self.sma_options.clone())
                                        .shift(lit(-(self.sma_options.window_size as i32)))
                                        .alias(&avg_loss)
                        ])
                        .collect()?;

                    updated_df = updated_df.clone()
                        .lazy()
                        .with_column(
                            (col(&avg_gain) / col(&avg_loss)).alias(&rs)
                        )
                        .collect()?;

                    updated_df = updated_df.clone()
                        .lazy()
                        .with_column(
                            (lit(100.0) - (lit(100.0) / (lit(1.0) + col(&rs)))).alias(&rsi)
                        )
                        .collect()?;
                    self.df = Some(updated_df);
                    info!("Calculated RSI");
                    return Ok(());
                }
                None => {
                    debug!("DataFrame is None");
                    return Err("DataFrame is None".into());
                }
            }
        }
    pub fn calc_signal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.df {
            Some(df) => {
                let columns = df.get_column_names();
                let rsi_col = columns.iter()
                    .find(|name| name.contains("RSI"))
                    .ok_or("RSI column not found")?;
                let signal_name = format!("Sig_{}", self.sma_options.window_size);
                self.df = df.clone()
                    .lazy()
                    .with_column(
                        when(col(rsi_col.to_string()).gt(lit(self.upper_bound as u32)))
                            .then(-1)
                            .when(col(rsi_col.to_string()).lt(lit(self.lower_bound as u32)))
                            .then(1)
                            .otherwise(0)
                            .alias(signal_name)
                    )
                    .collect().ok();
                return Ok(());                        
            }
            None => {
                debug!("DataFrame is None");
                return Err("DataFrame is None".into());
            }
        }
    }
}
