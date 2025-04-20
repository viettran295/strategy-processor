use polars::prelude::*;
use log::info;

pub struct CrossingAvg {
    pub ma_type: String,
    pub df: Option<DataFrame>,
    pub ma_options: RollingOptionsFixedWindow,
}

impl CrossingAvg {
    pub fn new(df: DataFrame) -> Self {
        let ma_options = RollingOptionsFixedWindow {
            window_size: 1,
            min_periods: 1,
            weights: None,
            center: false,
            fn_params: None,
        };
        CrossingAvg { 
            ma_type: "SMA".to_string(),
            df: Some(df),
            ma_options,
        }
    }
}

impl CrossingAvg {
    pub fn calc_ma(&mut self, window_size: usize) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.df {
            Some(df) => {
                let ma_name = format!("{}_{}", self.ma_type, window_size);
                self.ma_options.window_size = window_size;
                // Implementation of calculating signal for moving average strategy
                self.df = df.clone()
                    .lazy()
                    .with_column(
                        col("close").rolling_mean(self.ma_options.clone()).alias(ma_name.clone()),
                    )
                    .collect().ok();
                info!("Calculated {}", ma_name);
                return Ok(());
            },
            None => return Err("Dataframe is None".into())
        }
    }
    pub fn calc_signal(
                &mut self, 
                short_ma: usize, 
                long_ma: usize
            ) -> Result<(), Box<dyn std::error::Error>> {
                if self.df.is_none() {
                    return Err("Dataframe is None".into());
                }

                let signal_name = format!("Sig_{}_{}_{}", self.ma_type, short_ma, long_ma);
                let short_ma_name = format!("{}_{}", self.ma_type, short_ma);
                let long_ma_name = format!("{}_{}", self.ma_type, long_ma);

                // Calculate MAs up front
                if ! self.df.as_ref().unwrap().column(short_ma_name.as_str()).is_ok() {
                    self.calc_ma(short_ma)?;
                }
                if ! self.df.as_ref().unwrap().column(long_ma_name.as_str()).is_ok() {
                    self.calc_ma(long_ma)?;
                }

                // Get reference to current dataframe
                let df = self.df.as_ref().unwrap();
                
                self.df = df.clone()
                    .lazy()
                    .with_columns([
                        when(
                            col(&short_ma_name).gt(col(&long_ma_name)).and(
                                col(&short_ma_name).shift(lit(1)).gt(col(&long_ma_name))
                            )
                        )
                        .then(lit(1))
                        .when(
                            col(&short_ma_name).lt(col(&long_ma_name)).and(
                                col(&short_ma_name).shift(lit(1)).gt_eq(col(&long_ma_name))
                            )
                        )
                        .then(lit(-1))
                        .otherwise(lit(0))
                        .alias(&signal_name)
                    ])
                    .collect().ok();
                info!("Calculated crossing average signal: {}", signal_name);
                Ok(())
            }
}