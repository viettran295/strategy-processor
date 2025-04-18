mod config;
mod fetch;
mod processor;

use actix_web::{web::{self, Query}, App, HttpResponse, HttpServer};
use env_logger;
use log::error;
use fetch::StockFetcher;
use serde::Deserialize;

#[derive(Deserialize)]
struct QueryParams {
    start_date: Option<String>,
    end_date: Option<String>,
}

async fn get_price(
    symbol: web::Path<String>, 
    query: Query<QueryParams>
) -> HttpResponse {
    let fetcher = StockFetcher::new();
    let start_date = query.start_date.clone();
    let end_date = query.end_date.clone();

    match fetcher.fetch_prices(symbol.to_string(), start_date, end_date).await {
        Ok(stock_data) => {
            return HttpResponse::Ok()
                .content_type("application/json")
                .body(stock_data);
        }
        Err(e) => {
            error!("Error getting stock prices: {}", e);
            return HttpResponse::InternalServerError()
                .body(format!("Error getting stock prices: {}", e));
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .route("/{symbol}", web::get().to(get_price))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}