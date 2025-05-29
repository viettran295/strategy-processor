use log::{debug, error};
use regex::Regex;

use crate::processor::{Strategy, StrategyRSI};

use super::{Backtest, ScannerPerformance};

pub struct ScannerRSI {
    strategy: StrategyRSI,
    from_ma: usize,
    to_ma: usize,
    sig_col: String,
    backtest: Backtest
}

impl ScannerRSI {
    pub fn new(strategy: StrategyRSI, from_ma: usize, to_ma: usize) -> Self {
        ScannerRSI {
            strategy: strategy,
            from_ma: from_ma,
            to_ma: to_ma,
            sig_col: String::from("Sig"),
            backtest: Backtest::new(),
        }
    }
}

impl ScannerPerformance for ScannerRSI {
    fn scan_performance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let step = 2;
        for ma_window in (self.from_ma..(self.to_ma)).step_by(step) {
            self.strategy.update_params(ma_window, 80, 20);
            self.strategy.calc_rsi()?;
            self.strategy.calc_signal()?;
        }
        for col in self.strategy.df.as_ref().unwrap().get_column_names() {
            if col.contains(self.sig_col.as_str()) {
                self.backtest.execute(self.strategy.df.as_ref().unwrap(), col);
            }
        }
        return Ok(());
    }
    
    fn best_performance(&mut self) -> Option<(&String, &f32)> {
        if self.backtest.results.is_empty() {
            match self.scan_performance() {
                Ok(_) => {}
                Err(e) => {
                    error!("Error scanning best performance: {}", e);
                    return None;
                }
            }
        } 
        let res = self.backtest.results.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap());
        return res;
    }
    
    fn get_best_performance_df(&mut self) -> Option<polars::prelude::DataFrame> {
        let best_perf = self.best_performance()?;
        let best_perf_col = best_perf.0.clone();
        let re = Regex::new(r"_(\d+)").expect("Failed to extract long short MA");

        if let Some(captures) = re.captures(best_perf_col.as_str()) {            
            let ma_window = captures.get(1).unwrap().as_str();
            let rsi_col = format!("RSI_{}", ma_window);
            let df = self.strategy.df.as_ref()?;
            let cols = vec!["datetime".to_string(), "high".to_string(), "low".to_string(),
                                        "open".to_string(), "close".to_string(), rsi_col, best_perf_col];
            match df.select(cols) {
                Ok(df) => Some(df),
                Err(_) => None,
            }
        } else {
            debug!("Error getting best performance df");
            return None;
        }

    }
}