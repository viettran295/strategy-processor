mod crossing_ma;
mod rsi;
mod bollinger_bands;

pub use crossing_ma::StrategyCrossingMA;
pub use rsi::StrategyRSI;
pub use bollinger_bands::StrategyBollingerBands;

pub trait Strategy {
    fn calc_signal(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}