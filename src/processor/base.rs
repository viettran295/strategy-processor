use serde::{Deserialize, Serialize};

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
}

impl DfColumns {
    pub fn new() -> Self {
        DfColumns {
            column_names: Vec::new(),
            column_types: Vec::new()
        }
    }
}