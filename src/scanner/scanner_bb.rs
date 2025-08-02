use log::error;
use rayon::scope;
use polars::prelude::*;
use regex::Regex;
use std::sync::mpsc;

use crate::{scanner::{Backtest, ScannerPerformance}, strategy::{Strategy, StrategyBollingerBands}};

pub struct ScannerBollingerBands {
    strategy: StrategyBollingerBands,
    from_ma: usize,
    to_ma: usize,
    sig_col: String,
    backtest: Backtest
}

impl ScannerBollingerBands {
    pub fn new(strategy: StrategyBollingerBands, from_ma: usize, to_ma: usize) -> Self {
        ScannerBollingerBands{
            strategy: strategy,
            from_ma: from_ma,
            to_ma: to_ma,
            sig_col: String::from("Sig"),
            backtest: Backtest::new(),
        }
    }
}

impl ScannerPerformance for ScannerBollingerBands {
    fn scan_performance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let step = 5;
        let (tx, rx): (mpsc::Sender<DataFrame>, mpsc::Receiver<DataFrame>) = mpsc::channel();
        let strategy = self.strategy.clone();
        scope(|s| {
            for ma_window in (self.from_ma..(self.to_ma)).step_by(step) {
                for std in 2..4{
                    let mut strategy_clone = strategy.clone();
                    let tx_clone = tx.clone();
                    s.spawn(move |_| {
                        strategy_clone.update_param(Some(ma_window), Some(std));
                        let df = strategy_clone.calc_signal().unwrap();
                        tx_clone.send(df).unwrap();
                    });
                }
            }
        });
        drop(tx);
        for df in rx {
            for col in df.get_column_names() {
                if col.contains(self.sig_col.as_str()) {
                    self.backtest.execute(&df, col);
                }
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
        let re = Regex::new(r"SMA_(\d+)_Std_(\d+)").expect("Failed to extract parameters for Bollinger bands");
        
        if let Some(captures) = re.captures(best_perf_col.as_str()) {
            let ma_window = captures.get(1).unwrap().as_str();
            let std = captures.get(2).unwrap().as_str();
            let sma_col = format!("SMA_{}", ma_window);
            let upper_band = format!("Upper_SMA_{}_Std_{}", ma_window, std);
            let lower_band = format!("Lower_SMA_{}_Std_{}", ma_window, std);
            self.strategy.update_param(Some(ma_window.parse().unwrap()), Some(std.parse().unwrap()));
            let df = self.strategy.calc_signal().unwrap();
            let cols = vec!["datetime".to_string(), "high".to_string(), 
                            "low".to_string(), "open".to_string(), "close".to_string(), 
                            sma_col, upper_band, lower_band, best_perf_col];
            match df.select(cols) {
                Ok(df) => return Some(df),
                Err(_) =>  return None,
            }
        }
        error!("Fail to capture Bollinger bands parameters");
        return None;
    }
}
