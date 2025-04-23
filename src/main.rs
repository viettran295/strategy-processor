mod config;
mod fetch;
mod processor;
mod handler;

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
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}