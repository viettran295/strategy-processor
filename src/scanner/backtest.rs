use log::debug;
use polars::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Backtest{
    pub col_name: String,
    pub shares_hold: i32,
    pub results: HashMap<String, f32>
}

impl Backtest {
    pub fn new() -> Self {
        Backtest {
            col_name: String::from("Backtest"),
            shares_hold: 0,
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
            let mut cum_sum = 0.0;
            self.shares_hold = 0;
            for i in (0..df_bt.height()).rev() {
                if let Some(exec_price) = self.anyvalue_to_float(df_bt.column("Backtest").unwrap().get(i).unwrap()) {
                    let share = self.anyvalue_to_float(df_bt.column(sig_col).unwrap().get(i).unwrap()).unwrap();
                    if ((exec_price < 0.0) & (self.shares_hold == 0) ) ||
                        ((exec_price != 0.0) & (self.shares_hold > 0)) {
                        cum_sum += exec_price;
                        self.shares_hold -= share as i32;
                    }
                } else {
                    debug!("Unsupported value type in backtest column at row {}", i);
                }
            }
            self.results.insert(sig_col.to_string(), cum_sum);
        }
    }

    fn anyvalue_to_float(&self, av: AnyValue) -> Option<f32> {
        match av {
            AnyValue::Float64(bt_value) => Some(bt_value as f32),
            AnyValue::Float32(bt_value) => Some(bt_value),
            AnyValue::Int64(bt_value) => Some(bt_value as f32),
            AnyValue::Int32(bt_value) => Some(bt_value as f32),
            _ => None
        }
    }
}