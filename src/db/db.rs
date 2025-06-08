use polars::prelude::*;
use duckdb::{Connection, Result};
use std::{fs, path::Path};

#[derive(Debug)]
pub struct DbManager {
    data_dir: String,
    db_path: String
}

impl Default for DbManager {
    fn default() -> Self {
        let base_path = "data";
        let db_path = format!("{}/stocks.db", base_path);
        let data_dir = Path::new(base_path);
        if !data_dir.is_dir() {
            fs::create_dir_all(data_dir).expect("Error creating data dir");
        }
        DbManager {
            data_dir: data_dir.to_str().unwrap().to_string(),
            db_path: String::from(db_path)
        }
    }
}

impl DbManager {
    fn new(data_dir: String, db_path: String) -> Self {
        DbManager {
            data_dir: data_dir,
            db_path: db_path
        }
    }
    
    pub fn create_table(&self, table_name: String, df: &mut DataFrame) -> Result<()> {
        let parquet_file = format!("{}/{}.parquet", self.data_dir, table_name);
        let mut file = std::fs::File::create(parquet_file.clone()).expect("Failed to create file");
        ParquetWriter::new(&mut file)
            .with_compression(ParquetCompression::Snappy)
            .finish(df)
            .expect("Error writing parquet file");
        
        let conn = Connection::open(self.db_path.as_str())?;
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {} AS SELECT * FROM read_parquet('{}')", 
            table_name, 
            parquet_file
        );
        conn.execute(&query, [],)?;
        Ok(())
    }
    
    pub fn clean_up(&self) -> Result<()> {
        let entries = fs::read_dir(self.data_dir.as_str()).unwrap();
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "parquet" || extension == "db" {
                        fs::remove_file(&path).expect("Error cleaning database");
                    }
                }
            }
        }
        Ok(())
    }
}