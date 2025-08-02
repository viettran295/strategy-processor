mod config;
mod fetch;
mod strategy;
mod handler;
mod converter;
mod scanner;
mod db;
mod jobs;

use actix_web::{web, App, HttpServer};
use handler::*;
use jobs::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tokio::spawn(remove_cache_db());
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .route("/{symbol}", web::get().to(get_price))
            .route("/sma/{symbol}", web::get().to(get_sma_signal))
            .route("/ewma/{symbol}", web::get().to(get_ewma_signal))
            .route("/rsi/{symbol}", web::get().to(get_rsi_signal))
            .route("/bb/{symbol}", web::get().to(get_bb_signal))
            .route("/bestperf/sma/{symbol}", web::get().to(get_best_performance_sma))
            .route("/bestperf/ewma/{symbol}", web::get().to(get_best_performance_ewma))
            .route("/bestperf/rsi/{symbol}", web::get().to(get_best_performance_rsi))
            .route("/bestperf/bb/{symbol}", web::get().to(get_best_performance_bb))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}