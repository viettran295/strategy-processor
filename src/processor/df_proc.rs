use log::{error, info};
use polars::prelude::*;

#[derive(Clone, Debug)]
pub struct DfProcessor {
    pub df: Option<DataFrame>,
}

impl DfProcessor {
    pub fn new() -> Self {
        return Self {
            df: None
        }
    }
    pub fn to_df(&mut self, prices: Vec<f32>, datetime: &[&str]) {
        if prices.len() != datetime.len() {
            error!("Length of 2 columns must be equal");
            return
        }
        
        if let Ok(df) = DataFrame::new(vec![
            Series::new("close".into(), prices).into(),
            Series::new("datetime".into(), datetime).into(),
        ]) {
            self.df = Some(df);
            info!("Converted data to dataframe");
        } else {
            error!("Error converting dataframe")
        }
    }
}