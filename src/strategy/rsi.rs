use polars::prelude::*;
use log::{info, debug};
use serde::{Deserialize, Serialize};

use super::Strategy;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StrategyRSI {
    pub df: Option<DataFrame>,
    pub upper_bound: usize,
    pub lower_bound: usize,
    pub sma_options: RollingOptionsFixedWindow,
}

impl StrategyRSI {
    pub fn new(
        df: DataFrame, 
        period: usize,
        upper_bound: usize,
        lower_bound: usize
    ) -> Self {
        let sma_options = RollingOptionsFixedWindow {
            window_size: period,
            min_periods: 1,
            weights: None,
            center: false,
            fn_params: None,
        };
        StrategyRSI {
            df: Some(df),
            sma_options,
            upper_bound,
            lower_bound
        }
    }
    
    pub fn update_params(&mut self, period: Option<usize>, upper_bound: Option<usize>, lower_bound: Option<usize>) {
        if let Some(p) = period {
            self.sma_options.window_size = p;
        }
        if let Some(u) = upper_bound {
            self.upper_bound = u;
        }
        if let Some(l) = lower_bound {
            self.lower_bound = l;
        }
    }
    
    pub fn calc_rsi(&mut self) -> Result<DataFrame, Box<dyn std::error::Error>> {
        match &mut self.df {
            Some(df) => {
                let avg_gain = format!("avg_gain_{}", self.sma_options.window_size);
                let avg_loss = format!("avg_loss_{}", self.sma_options.window_size);
                let rs = format!("RS_{}", self.sma_options.window_size);
                let rsi = format!("RSI_{}", self.sma_options.window_size);
                let delta = "delta";
                let gain = "gain";
                let loss = "loss";

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
                        when(col(delta).gt(lit(0.0)))
                            .then(col(delta))
                            .otherwise(lit(0.0))
                            .alias(gain),
                        when(col(delta).lt(lit(0.0)))
                            .then(-col(delta))
                            .otherwise(lit(0.0))
                            .alias(loss),
                    ])
                    .collect()?;

                updated_df = updated_df.clone()
                    .lazy()
                    .with_columns([
                        col(gain).rolling_mean(self.sma_options.clone())
                                    .shift(lit(-(self.sma_options.window_size as i32)))
                                    .alias(&avg_gain),
                        col(loss).rolling_mean(self.sma_options.clone())
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
                
                updated_df = updated_df.drop_many(
                                vec![delta.to_string(), gain.to_string(), 
                                            loss.to_string(), avg_gain, avg_loss, rs]
                            );
                info!("Calculated {}", rsi);
                return Ok(updated_df);
            }
            None => {
                debug!("DataFrame is None");
                return Err("DataFrame is None".into());
            }
        }
    }
}

impl Strategy for StrategyRSI{
    fn calc_signal(&mut self) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let mut df = self.calc_rsi()?;
        let columns = df.get_column_names();
        let rsi_col_name = format!("RSI_{}", self.sma_options.window_size);
        let rsi_col = columns.iter()
            .find(|name| name.contains(&rsi_col_name))
            .ok_or("RSI column not found")?;
        let signal_name = format!("Sig_{}", self.sma_options.window_size);
        df = df.clone()
            .lazy()
            .with_column(
                when(col(rsi_col.to_string()).gt(lit(self.upper_bound as u32)))
                    .then(1)
                    .when(col(rsi_col.to_string()).lt(lit(self.lower_bound as u32)))
                    .then(-1)
                    .otherwise(0)
                    .alias(signal_name)
            )
            .collect().ok().unwrap();
        return Ok(df);                                 
    }
}
