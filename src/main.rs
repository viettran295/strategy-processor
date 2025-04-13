mod config;
mod fetch;
mod processor;

use actix_web::{HttpResponse, web, App, HttpServer};
use env_logger;
use log::error;
use fetch::StockFetcher;

async fn get_price(symbol: web::Path<String>) -> HttpResponse {
    let fetcher = StockFetcher::new();

    match fetcher.fetch_prices(symbol.to_string(), None, None).await {
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
    .bind("127.0.0.1:8000")?
    .run()
    .await
}