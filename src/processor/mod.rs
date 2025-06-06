mod base;
mod crossing_ma;
mod rsi;

pub use crossing_ma::{StrategyCrossingMA, CrossingMAResponse, CrossingMAData};
pub use base::DfColumns;
pub use rsi::{StrategyRSI, RSIResponse, RSIData};

pub trait Strategy {
    fn calc_signal(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}