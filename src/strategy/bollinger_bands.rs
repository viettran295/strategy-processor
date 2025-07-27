use polars::{prelude::*};
use log::info;
use serde::{Deserialize, Serialize};

use super::Strategy;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StrategyBollingerBands {
    pub ma_window: usize,
    pub std_bands: usize,
    pub df: Option<DataFrame>,
    pub sma_options: RollingOptionsFixedWindow,
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
    
    pub fn calc_ma(&mut self) -> Result<DataFrame, Box<dyn std::error::Error>> {
        match &mut self.df {
            Some(df) => {
                let ma_type = format!("SMA_{}", self.ma_window);
                let std_col = "Std";
                self.sma_options.window_size = self.ma_window;
                let mut df_result = df.clone()
                    .lazy()
                    .with_column(
                        col("close").rolling_mean(self.sma_options.clone())
                                    .alias(ma_type.clone())
                                    // Shift the calculated moving average to the corresponding datetime
                                    .shift((-(self.ma_window as i32)).into()),
                    )
                    .collect().ok().unwrap();
                df_result = df_result.clone()
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
                    .collect().ok().unwrap();
            info!("Calculated bollinger bands {}", ma_type.clone());

            return Ok(df_result);
        },
            None => return Err("Dataframe is None".into())
        }
    }
}

impl Strategy for StrategyBollingerBands {
    fn calc_signal(&mut self) -> Result<DataFrame, Box<dyn std::error::Error>> {
        if self.df.is_none() {
            return Err("Dataframe is None".into());
        }

        let signal_name = format!("Sig_SMA_{}_Std_{}", self.ma_window, self.std_bands);
        let upper_band_name = format!("Upper_{}_SMA_{}", self.std_bands, self.ma_window);
        let lower_band_name = format!("Lower_{}_SMA_{}", self.std_bands, self.ma_window);
        let mut df_result = self.calc_ma()?;
        
        df_result = df_result.clone()
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
            .collect().ok().unwrap();
        info!("Calculated bollinger bands signal: {}", signal_name);
        Ok(df_result)
    }
}