mod config;
mod fetch;
mod processor;

use actix_web::{web::{self, Query}, App, HttpResponse, HttpServer};
use env_logger;
use processor::{CrossingAvg, DfProcessor};
use log::error;
use fetch::StockFetcher;
use serde::Deserialize;

#[derive(Deserialize)]
struct DateParams {
    start_date: Option<String>,
    end_date: Option<String>,
}

#[derive(Deserialize)]
struct CrossingMAParams {
    short_ma: Option<usize>,
    long_ma: Option<usize>
}

async fn get_price(
    symbol: web::Path<String>, 
    query: Query<DateParams>
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

async fn get_ma_signal(
    symbol: web::Path<String>,
    query: Query<CrossingMAParams>
) -> HttpResponse {
        let fetcher = StockFetcher::new();
        match fetcher.fetch_prices(symbol.to_string(), None, None).await {
            Ok(stock_data) => {
                let mut df_proc = DfProcessor::new();
                let short_ma = query.short_ma.clone().unwrap_or(20);
                let long_ma = query.long_ma.clone().unwrap_or(50);
                match df_proc.to_df(stock_data) {
                    Ok(_) => {
                        let mut crs_avg = CrossingAvg::new(df_proc.df.unwrap());
                        match crs_avg.calc_signal(short_ma, long_ma) {
                            Ok(_) => {
                                let response = DfProcessor::df_to_json(&crs_avg.df.clone().unwrap());
                                return HttpResponse::Ok()
                                    .content_type("application/json")
                                    .body(response)
                            }
                            Err(e) => {
                                error!("Error calculating signal: {}", e);
                                return HttpResponse::InternalServerError()
                                    .body(format!("Error calculating signal: {}", e))
                            }
                        }
                    },
                    Err(e) => {
                        error!("Error processing data: {}", e);
                        return HttpResponse::InternalServerError()
                            .body(format!("Error processing data: {}", e))
                    }
                }
            }
            Err(e) => {
                error!("Error getting moving average stock prices: {}", e);
                HttpResponse::InternalServerError()
                    .body(format!("Error getting moving avgerage stock prices: {}", e))
            }
        }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .route("/{symbol}", web::get().to(get_price))
            .route("/ma/{symbol}", web::get().to(get_ma_signal))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}