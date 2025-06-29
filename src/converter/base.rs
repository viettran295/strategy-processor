use serde::{Deserialize, Serialize};
use polars::prelude::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct DfBaseData {
    pub datetime: String,
    pub high: String,
    pub low: String,
    pub open: String,
    pub close: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DfColumns {
    pub column_names: Vec<String>,
    pub column_types: Vec<String>,
}

impl DfBaseData {
    pub fn new() -> Self {
        DfBaseData {
            datetime: String::new(),
            high: String::new(),
            low: String::new(),
            open: String::new(),
            close: String::new(),
        }
    }
    pub fn set_base_data(&mut self, df: &DataFrame, col: &Column, row: usize) {
        let base_type = col.name().as_str();
        let mut value = String::new();
        if base_type == "datetime" {
            value = df.column(col.name())
                                    .unwrap()
                                    .str()
                                    .unwrap()
                                    .get(row)
                                    .unwrap()
                                    .to_string();
        } else {
            value = df.column(col.name())
                            .unwrap()
                            .f32()
                            .unwrap()
                            .get(row)
                            .unwrap_or(0.0)
                            .to_string();
        }
        match base_type {
            "high" => self.high = value,
            "low" => self.low = value,
            "open" => self.open = value,
            "close" => self.close = value,
            "datetime" => self.datetime = value,
            _ => {}
        }
    }
}

impl DfColumns {
    pub fn new() -> Self {
        DfColumns {
            column_names: Vec::new(),
            column_types: Vec::new()
        }
    }
}