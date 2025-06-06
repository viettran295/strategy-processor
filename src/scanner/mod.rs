mod backtest;
mod scanner_ma;
mod scanner_rsi;

use polars::prelude::*;

pub use backtest::Backtest;
pub use scanner_ma::ScannerCrossingMA;
pub use scanner_rsi::ScannerRSI;

pub trait ScannerPerformance {
    fn scan_performance(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn best_performance(&mut self) -> Option<(&String, &f32)>;
    fn get_best_performance_df(&mut self) -> Option<DataFrame>;
}