mod config;
mod fetch;
mod processor;
mod handler;
mod converter;
mod scanner;
mod db;

use actix_web::{web, App, HttpServer};
use handler::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .route("/{symbol}", web::get().to(get_price))
            .route("/sma/{symbol}", web::get().to(get_sma_signal))
            .route("/ewma/{symbol}", web::get().to(get_ewma_signal))
            .route("/rsi/{symbol}", web::get().to(get_rsi_signal))
            .route("/bestperf/sma/{symbol}", web::get().to(get_best_performance_sma))
            .route("/bestperf/ewma/{symbol}", web::get().to(get_best_performance_ewma))
            .route("/bestperf/rsi/{symbol}", web::get().to(get_best_performance_rsi))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}