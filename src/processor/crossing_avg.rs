use polars::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct CrossingAvg {
    pub ma_type: String,
    pub df: Option<DataFrame>,
    pub ma_options: RollingOptionsFixedWindow,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DfColumns {
    pub column_names: Vec<String>,
    pub column_types: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DfData {
    pub datetime: String,
    pub high: String,
    pub low: String,
    pub open: String,
    pub close: String,
    pub short_ma: String,
    pub long_ma: String,
    pub signal: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CrossingMAResponse {
    pub columns: DfColumns,
    pub data: Vec<DfData>,
}

impl DfColumns {
    pub fn new() -> Self {
        DfColumns {
            column_names: Vec::new(),
            column_types: Vec::new()
        }
    }
}

impl DfData {
    pub fn new() -> Self {
        DfData { 
            datetime: String::new(),
            high: String::new(),
            low: String::new(),
            open: String::new(),
            close: String::new(),
            short_ma: String::new(),
            long_ma: String::new(),
            signal: String::new()
        }
    }
}

impl CrossingMAResponse {
    pub fn new(df_columns: DfColumns, df_data: Vec<DfData>) -> Self {
        CrossingMAResponse {
            columns: df_columns,
            data: df_data
        }
    }
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
    pub fn calc_ma(&mut self, window_size: usize, ma_name: String) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.df {
            Some(df) => {
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
                let short_ma_name = format!("{}_short_{}", self.ma_type, short_ma);
                let long_ma_name = format!("{}_long_{}", self.ma_type, long_ma);

                // Calculate MAs up front
                if ! self.df.as_ref().unwrap().column(short_ma_name.as_str()).is_ok() {
                    self.calc_ma(short_ma, short_ma_name.clone())?;
                }
                if ! self.df.as_ref().unwrap().column(long_ma_name.as_str()).is_ok() {
                    self.calc_ma(long_ma, long_ma_name.clone())?;
                }

                // Get reference to current dataframe
                let df = self.df.as_ref().unwrap();
                
                self.df = df.clone()
                    .lazy()
                    .with_columns([
                        when(
                            col(&short_ma_name).gt(col(&long_ma_name)).and(
                                col(&short_ma_name).shift(lit(1)).lt_eq(col(&long_ma_name))
                            )
                        )
                        .then(lit(-1))
                        .when(
                            col(&short_ma_name).lt(col(&long_ma_name)).and(
                                col(&short_ma_name).shift(lit(1)).gt_eq(col(&long_ma_name))
                            )
                        )
                        .then(lit(1))
                        .otherwise(lit(0))
                        .alias(&signal_name)
                    ])
                    .collect().ok();
                info!("Calculated crossing average signal: {}", signal_name);
                Ok(())
            }
}