use log::debug;

use crate::processor::Strategy;
use crate::processor::{StrategyCrossingMA, StrategyRSI};

use super::Backtest;

pub trait ScannerPerformance {
    fn scan_performance(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn best_performance(&mut self) -> Option<(&String, &f32)>;
}

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
        let mut res: Option<(&String, &f32)> = None;
        if !self.backtest.results.is_empty() {
            res = self.backtest.results.iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap());
        } else {
            debug!("No backtest results found");
        }
        return res;
    }
}