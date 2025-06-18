use polars::{prelude::*};
use log::info;
use serde::{Deserialize, Serialize};

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
                        ((col(ma_type.clone()) + col(std_col)) * lit(self.std_bands as f32))
                            .alias(format!("Upper_{}_SMA_{}", self.std_bands, self.ma_window))
                    )
                    .with_column(
                        ((col(ma_type.clone()) - col(std_col)) * lit(self.std_bands as f32))
                            .alias(format!("Lower_{}_SMA_{}", self.std_bands, self.ma_window))
                    )
                    .collect().ok();
            println!("{:?}", self.df);
            let filtered_df = self.df.as_ref().unwrap().filter(&self.df.as_ref().unwrap().column("Std")?.is_not_null())?;
            println!("{:?}", filtered_df);
            info!("Calculated bollinger bands {}", ma_type.clone());

            return Ok(());
        },
            None => return Err("Dataframe is None".into())
        }
    }
}