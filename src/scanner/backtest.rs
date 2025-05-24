use log::debug;
use polars::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Backtest{
    pub col_name: String,
    pub results: HashMap<String, f32>
}

impl Backtest {
    pub fn new() -> Self {
        Backtest {
            col_name: String::from("Backtest"),
            results: HashMap::new()
        }
    }
    pub fn execute(&mut self, df: &DataFrame, sig_col: &str) {
        let df_bt = df.clone()
            .lazy()
            .with_column(
                (col("close") * col(sig_col)).alias(self.col_name.as_str())
            )
            .collect().ok();
        if let Some(df_bt) = df_bt{
            // let sum = df_bt.column("backtest").unwrap().f64().unwrap().sum().unwrap();
            let mut cum_sum = 0.0;
            let mut trades = 0;
            let mut last_exec_price = 0.0;
            let mut first_trade: bool = true;
            for i in (0..df_bt.height()).rev() {
                if let Some(exec_price) = self.get_exec_price(df_bt.column("Backtest").unwrap().get(i).unwrap()) {
                    if exec_price > 0.0 && first_trade {
                        first_trade = false;
                        continue;
                    }
                    if exec_price != 0.0 {
                        cum_sum += exec_price;
                        trades += 1;
                        last_exec_price = exec_price;
                    }
                } else {
                    debug!("Unsupported value type in backtest column at row {}", i);
                }
            }
            // Trading pair must be even
            if trades % 2 != 0 {
                cum_sum -= last_exec_price;
            }
            self.results.insert(sig_col.to_string(), cum_sum);
            debug!("Backtest result: {:?}", self.results);
        }
    }
    fn get_exec_price(&self, av: AnyValue) -> Option<f32> {
        match av {
            AnyValue::Float64(bt_value) => Some(bt_value as f32),
            AnyValue::Float32(bt_value) => Some(bt_value),
            _ => None
        }
    }
}