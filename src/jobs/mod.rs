use std::{str::FromStr, thread};
use chrono::Utc;
use cron::Schedule;
use log::debug;

use crate::db::DbManager;

pub async fn remove_cache_db() {
    let db = DbManager::default();
    let schedule = Schedule::from_str("0 7 * * * *").unwrap();
    let mut upcoming = schedule.upcoming(Utc);
    loop {
        if let Some(next) = upcoming.next() {
            let db_ref = db.clone();
            let now = Utc::now();
            let sleep_time = (next - now).to_std().unwrap();
            thread::sleep(sleep_time);
            tokio::task::spawn_blocking(move || db_ref.clean_up().unwrap());
            
            debug!("Cache db is cleaned at {}", next);
        }
    }
}