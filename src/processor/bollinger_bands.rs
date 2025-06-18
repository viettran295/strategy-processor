use polars::{prelude::*};
use log::{info, debug};
use serde::{Deserialize, Serialize};
use crate::processor::base::{DfBaseData, DfColumns};

use super::Strategy;

#[derive(Deserialize, Serialize, Debug)]
pub struct BollingerBandsData {
    #[serde(flatten)]
    pub base_data: DfBaseData,
    pub ma_windows: String,
    pub upper_band: String,
    pub lower_band: String,
    pub signal: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BollingerBandsResponse {
    pub columns: DfColumns,
    pub data: Vec<BollingerBandsData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StrategyBollingerBands {
    pub ma_window: usize,
    pub std_bands: usize,
    pub df: Option<DataFrame>,
    pub sma_options: RollingOptionsFixedWindow,
}

impl BollingerBandsData {
    pub fn new() -> Self {
        let base_data = DfBaseData::new();
        BollingerBandsData { 
            base_data,
            ma_windows: String::new(),
            upper_band: String::new(),
            lower_band: String::new(),
            signal: String::new()
        }
    }
}

impl BollingerBandsResponse {
    pub fn new(df_columns: DfColumns, df_data: Vec<BollingerBandsData>) -> Self {
        BollingerBandsResponse {
            columns: df_columns,
            data: df_data
        }
    }
}

impl StrategyBollingerBands {
    pub fn new(df: DataFrame, ma_window: usize) -> Self {
        let sma_options = RollingOptionsFixedWindow {
            window_size: 3,
            min_periods: 1,
            weights: None,
            center: false,
            fn_params: None,
        };
        StrategyBollingerBands {
            df: Some(df),
            ma_window: ma_window,
            std_bands: 2,
            sma_options,
        }
    }
    
    pub fn calc_ma(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.df {
            Some(df) => {
                let ma_type = format!("SMA_{}", self.ma_window);
                let std_col = "Std";
                self.sma_options.window_size = self.ma_window;
                self.df = df.clone()
                    .lazy()
                    .with_column(
                        col("close").rolling_mean(self.sma_options.clone())
                                    .alias(ma_type.clone())
                                    // Shift the calculated moving average to the corresponding datetime
                                    .shift((-(self.ma_window as i32)).into()),
                    )
                    .collect().ok();
                self.df = self.df.as_ref()
                    .unwrap()
                    .clone()
                    .lazy()
                    .with_column(
                        col(ma_type.clone()).rolling_std(self.sma_options.clone()).alias(std_col)
                    )
                    .with_column(
                        (col(ma_type.clone()) + col(std_col) * lit(self.std_bands as f32))
                            .alias(format!("Upper_{}_SMA_{}", self.std_bands, self.ma_window))
                    )
                    .with_column(
                        (col(ma_type.clone()) - col(std_col) * lit(self.std_bands as f32))
                            .alias(format!("Lower_{}_SMA_{}", self.std_bands, self.ma_window))
                    )
                    .collect().ok();
            info!("Calculated bollinger bands {}", ma_type.clone());

            return Ok(());
        },
            None => return Err("Dataframe is None".into())
        }
    }
}

impl Strategy for StrategyBollingerBands {
    fn calc_signal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.df.is_none() {
            return Err("Dataframe is None".into());
        }

        let signal_name = format!("Sig_SMA_{}_Std_{}", self.ma_window, self.std_bands);
        let upper_band_name = format!("Upper_{}_SMA_{}", self.std_bands, self.ma_window);
        let lower_band_name = format!("Lower_{}_SMA_{}", self.std_bands, self.ma_window);

        // Calculate MAs up front
        if ! self.df.as_ref().unwrap().column(upper_band_name.as_str()).is_ok() || 
            ! self.df.as_ref().unwrap().column(lower_band_name.as_str()).is_ok() {
            self.calc_ma()?;
        }
        // Get reference to current dataframe
        let df = self.df.as_ref().unwrap();
        
        self.df = df.clone()
            .lazy()
            .with_columns([
                // Sell signal
                when(
                    col("close").gt(col(&upper_band_name))
                )
                .then(lit(1))
                // Buy signal
                .when(
                    col("close").lt(col(&lower_band_name))
                )
                .then(lit(-1))
                .otherwise(lit(0))
                .alias(&signal_name)
            ])
            .collect().ok();
        info!("Calculated bollinger bands signal: {}", signal_name);
        Ok(())
    }
}