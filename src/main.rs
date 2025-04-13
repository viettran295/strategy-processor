mod config;
mod fetch;
mod processor;

use actix_web::{HttpResponse, web, App, HttpServer};
use env_logger;
use log::error;
use fetch::StockFetcher;
use processor::DfProcessor;

async fn get_price(symbol: web::Path<String>) -> HttpResponse {
    let fetcher = StockFetcher::new();

    match fetcher.fetch_prices(symbol.to_string()).await {
        Ok(stock_data) => {
            let mut df_proc = DfProcessor::new();

            let close_prices: Result<Vec<f32>, _> = stock_data.iter()
                .map(|p| p.close.parse::<f32>())
                .collect();
            let close_prices: Vec<f32> = match close_prices {
                Ok(prices) => prices,
                Err(_) => {
                    error!("Failed to parse close prices");
                    return HttpResponse::InternalServerError()
                        .body("Failed to parse close prices");
                }
            };

            let datetime: Vec<&str> = stock_data.iter()
                .map(|d| d.datetime.as_str())
                .collect();

            df_proc.to_df(close_prices, &datetime);
            if let Some(df) = &df_proc.df {
                let json_response = DfProcessor::df_to_json(&df);
                return HttpResponse::Ok()
                    .content_type("application/json")
                    .body(json_response);
            } else {
                return HttpResponse::InternalServerError()
                    .body("Failed to get Dataframe")
            }
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