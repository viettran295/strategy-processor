use polars::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};

use super::Strategy;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StrategyCrossingMA {
    pub ma_type: String,
    pub short_ma: usize,
    pub long_ma: usize,
    pub df: Option<DataFrame>,
    pub sma_options: RollingOptionsFixedWindow,
    pub ewma_options: EWMOptions,
}

impl StrategyCrossingMA {
    pub fn new(df: DataFrame, short_ma: usize, long_ma: usize, ma_type: String) -> Self {
        let sma_options = RollingOptionsFixedWindow {
            window_size: 3,
            min_periods: 1,
            weights: None,
            center: false,
            fn_params: None,
        };
        let ewma_options = EWMOptions {
            alpha: 1.0,
            adjust: true,
            bias: false,
            min_periods: 1,
            ignore_nulls: true,
        };
        StrategyCrossingMA { 
            df: Some(df),
            ma_type: ma_type,
            short_ma: short_ma,
            long_ma: long_ma,
            sma_options,
            ewma_options
        }
    }
    
    pub fn update_params(&mut self, short_ma: Option<usize>, long_ma: Option<usize>, ma_type: Option<String>) {
        if let Some(s_ma) = short_ma {
            self.short_ma = s_ma;
        }
        if let Some(l_ma) = long_ma {
            self.long_ma = l_ma;
        }
        if let Some(ma_type) = ma_type {
            self.ma_type = ma_type;
        }
    }
    
    pub fn calc_ma(&mut self, window_size: usize, ma_name: String) -> Result<DataFrame, Box<dyn std::error::Error>> {
        match &mut self.df {
            Some(df) => {
                let mut df_result = DataFrame::empty();
                // Implementation of calculating signal for moving average strategy
                if self.ma_type == "SMA" {
                    self.sma_options.window_size = window_size;
                    self.sma_options.min_periods = window_size;
                    df_result = df.clone()
                        .lazy()
                        .with_column(
                            col("close").rolling_mean(self.sma_options.clone())
                                        .alias(ma_name.clone())
                                        // Shift the calculated moving average to the corresponding datetime
                                        .shift((-(window_size as i32)).into()),
                        )
                        .collect().ok().unwrap();
                } else if self.ma_type == "EWMA" {
                    self.ewma_options.alpha = 2.0 / (window_size + 1) as f64;
                    self.ewma_options.min_periods = window_size;
                    df_result = df.clone()
                        .lazy()
                        .with_column(
                            col("close").ewm_mean(self.ewma_options.clone())
                                        .alias(ma_name.clone())
                                        .shift((-(window_size as i32)).into()),
                        )
                        .collect().ok().unwrap();
                }
                info!("Calculated {}", ma_name);
                return Ok(df_result);
            },
            None => return Err("Dataframe is None".into())
        }
    }
}

impl Strategy for StrategyCrossingMA {
    fn calc_signal(&mut self) -> Result<DataFrame, Box<dyn std::error::Error>> {
        if self.df.is_none() {
            return Err("Dataframe is None".into());
        }

        let signal_name = format!("Sig_{}_{}_{}", self.ma_type, self.short_ma, self.long_ma);
        let short_ma_name = format!("{}_{}", self.ma_type, self.short_ma);
        let long_ma_name = format!("{}_{}", self.ma_type, self.long_ma);
        let shift_days = 1;

        // Calculate MAs up front
        let mut short_ma_df = self.calc_ma(self.short_ma, short_ma_name.clone())?;
        let long_ma_df = self.calc_ma(self.long_ma, long_ma_name.clone())?;
        let mut df_result = short_ma_df.with_column(
            long_ma_df.column(long_ma_name.as_ref())?.clone()
        ).unwrap().clone();
        
        df_result = df_result
            .lazy()
            .with_columns([
                // Sell signal
                when(
                    col(&short_ma_name).gt(col(&long_ma_name)).and(
                        col(&short_ma_name).shift(lit(shift_days)).lt(col(&long_ma_name).shift(lit(shift_days)))
                    )
                )
                .then(lit(1))
                // Buy signal
                .when(
                    col(&short_ma_name).lt(col(&long_ma_name)).and(
                        col(&short_ma_name).shift(lit(shift_days)).gt(col(&long_ma_name).shift(lit(shift_days)))
                    )
                )
                .then(lit(-1))
                .otherwise(lit(0))
                .alias(&signal_name)
            ])
            .collect()?;
        info!("Calculated crossing average signal: {}", signal_name);
        Ok(df_result)
    }
}