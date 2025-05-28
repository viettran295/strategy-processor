use log::{debug, error};
use polars::prelude::*;
use regex::Regex;

use crate::processor::{Strategy, StrategyCrossingMA};

use super::{Backtest, ScannerPerformance};

pub struct ScannerCrossingMA{
    strategy: StrategyCrossingMA,
    from_ma: usize,
    to_ma: usize,
    sig_col: String,
    backtest: Backtest,
}

impl ScannerCrossingMA {
    pub fn new(strategy: StrategyCrossingMA, from_ma: usize, to_ma: usize) -> Self {
        ScannerCrossingMA {
            strategy: strategy,
            from_ma: from_ma,
            to_ma: to_ma,
            sig_col: String::from("Sig"),
            backtest: Backtest::new(),
        }
    }
}

impl ScannerPerformance for ScannerCrossingMA {
    fn scan_performance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let step = 10;
        for short_ma in (self.from_ma..(self.to_ma - 1)).step_by(step) {
            for long_ma in ((short_ma + step)..self.to_ma).step_by(step) {
                self.strategy.update_params(short_ma, long_ma, self.strategy.ma_type.clone());
                self.strategy.calc_signal()?;
            };
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
    
    fn get_best_performance_df(&mut self) -> Option<DataFrame> {
        let best_perf = self.best_performance()?;
        let best_perf_col = best_perf.0.clone();
        let re = Regex::new(r"_(\d+)_(\d+)").expect("Failed to extract long short MA");
        
        if let Some(captures) = re.captures(best_perf_col.as_str()) {
            let short_win = captures.get(1).unwrap().as_str();
            let short_ma = format!("{}_{}", self.strategy.ma_type, short_win);
            let long_win = captures.get(2).unwrap().as_str();
            let long_ma = format!("{}_{}", self.strategy.ma_type, long_win);
            
            let df = self.strategy.df.as_ref()?;
            let cols = vec!["datetime".to_string(), "high".to_string(), "low".to_string(),
                            "open".to_string(), "close".to_string(), 
                            short_ma.to_string(), long_ma.to_string(), best_perf_col];
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
