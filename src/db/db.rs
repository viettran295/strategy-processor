use indexmap::IndexMap;
use polars::{frame::{row::AnyValueBuffer}, prelude::*};
use duckdb::{Connection, Result};
use std::{fs, path::Path, sync::{Mutex, MutexGuard}};
use log::{error, debug};

#[derive(Debug)]
pub struct DbManager {
    data_dir: String,
    db_path: Arc<Mutex<String>>
}

impl Default for DbManager {
    fn default() -> Self {
        let base_path = "data";
        let db_path = format!("{}/stocks.db", base_path);
        let data_dir = Path::new(base_path);
        if !data_dir.is_dir() {
            fs::create_dir_all(data_dir)
                .map_err(|e| error!("Error creating data dir: {}", e));
        }
        DbManager {
            data_dir: data_dir.to_str().unwrap().to_string(),
            db_path: Arc::new(Mutex::new(String::from(db_path))) 
        }
    }
}

impl DbManager {
    fn new(data_dir: String, db_path: String) -> Self {
        DbManager {
            data_dir: data_dir,
            db_path: Arc::new(Mutex::new(db_path))
        }
    }
    
    fn acquire_db(&self) -> anyhow::Result<MutexGuard<'_, String>> {
        self.db_path.lock()
            .map_err(|e| anyhow::anyhow!("Error acquiring db: {}", e))
    }

    pub fn create_table(&self, table_name: String, df: &mut DataFrame) -> Result<()> {
        let db_guard = self.acquire_db().unwrap();
        let parquet_file = format!("{}/{}.parquet", self.data_dir, table_name);
        let mut file = std::fs::File::create(parquet_file.clone()).expect("Failed to create file");
        ParquetWriter::new(&mut file)
            .with_compression(ParquetCompression::Snappy)
            .finish(df)
            .expect("Error writing parquet file");
        
        let conn = Connection::open(db_guard.clone().as_str())?;
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {} AS SELECT * FROM read_parquet('{}')", 
            table_name, 
            parquet_file
        );
        conn.execute(&query, [],)?;
        debug!("Created {} table", table_name);
        Ok(())
    }
    
    pub fn table_exists(&self, table_name: String) -> Result<bool> {
        let db_guard = self.acquire_db().unwrap();
        let conn = Connection::open(db_guard.clone().as_str())?;
        let query = format!(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_name = '{}'
            ) AS table_exists",
            table_name
        );
        let mut stmt = conn.prepare(&query)?;
        let table_exists = stmt.query_row([], |row| row.get(0))?;
        Ok(table_exists)
    }
    
    pub fn get_table(&self, table_name: String) -> Result<DataFrame> {
        let mut series_vec: Vec<Series> = Vec::new();
        let column_names_types = self.get_cols_names_types(table_name.clone())?;
        let mut buffer: Vec<AnyValueBuffer> = self.create_buffer(column_names_types.values().cloned().collect());

        let db_guard = self.acquire_db().unwrap();
        let conn = Connection::open(db_guard.clone().as_str())?;
        let query = format!("SELECT * FROM {}", table_name);
        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            for (i, _) in column_names_types.iter().enumerate() {
                let value: duckdb::types::Value = row.get(i)?;
                let cvt_val = match value {
                    duckdb::types::Value::Float(v) => AnyValue::Float32(v),
                    duckdb::types::Value::Int(v) => AnyValue::Int32(v),
                    duckdb::types::Value::Text(v) => AnyValue::StringOwned(v.into()),
                    _ => AnyValue::Null
                };
                buffer[i].add(cvt_val);
            }
        }
        for (i, name) in column_names_types.keys().enumerate() {
            let series = buffer[i].clone().into_series().with_name(name.into());
            series_vec.push(series);
        }
        let df = DataFrame::new(series_vec
                                .into_iter()
                                .map(|s| s.into()).collect::<Vec<Column>>()).unwrap();
        debug!("Queried table {}", table_name);
        Ok(df)
    }

    pub fn clean_up(&self) -> Result<()> {
        let db_guard = self.acquire_db().unwrap();
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
        debug!("All data is clean");
        Ok(())
    }

    fn get_cols_names_types(&self, table_name: String) -> Result<IndexMap<String, String>> {
        let db_guard = self.acquire_db().unwrap();
        let conn = Connection::open(db_guard.clone().as_str())?;
        let query_cols_names = format!("PRAGMA table_info({})", table_name);
        let mut cols_name_stmt = conn.prepare(&query_cols_names)?;
        let rows = cols_name_stmt.query_map([], |row| {
            let mut map: Vec<String> = Vec::new();
            map.push(row.get(1)?);
            map.push(row.get(2)?);
            Ok(map)
        })?;
        // Use IndexMap to preserve insertion order
        let mut column_names: IndexMap<String, String> = IndexMap::new();
        for row in rows {
            let row = row?;
            column_names.insert(row[0].clone(), row[1].clone());
        }
        Ok(column_names)
    }

    fn create_buffer(&self, buffer_types: Vec<String>) -> Vec<AnyValueBuffer> {
        let mut buffer: Vec<AnyValueBuffer> = Vec::new();
        for col_type in &buffer_types {
            if col_type.contains("VARCHAR") {
                buffer.push(AnyValueBuffer::new(&DataType::String, 0));
            } else if col_type.contains("FLOAT") {
                buffer.push(AnyValueBuffer::new(&DataType::Float32, 0));
            } else if col_type.contains("INT") {
                buffer.push(AnyValueBuffer::new(&DataType::Int32, 0));
            }
        } 
        return buffer;
    }
}