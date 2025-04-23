use actix_web::{web::{self, Query}, HttpResponse};
use log::error;
use serde::Deserialize;

use crate::fetch::StockFetcher;
use crate::processor::{CrossingAvg, DfProcessor};

#[derive(Deserialize)]
pub struct DateParams {
    start_date: Option<String>,
    end_date: Option<String>,
}

#[derive(Deserialize)]
pub struct CrossingMAParams {
    short_ma: Option<usize>,
    long_ma: Option<usize>
}

pub async fn get_price(
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

pub async fn get_ma_signal(
    symbol: web::Path<String>,
    ma_type: &str,
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
                        let mut crs_avg = CrossingAvg::new(df_proc.df.unwrap(), ma_type);
                        match crs_avg.calc_signal(short_ma, long_ma, ma_type) {
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

pub async fn get_sma_signal(
    symbol: web::Path<String>, 
    query:Query<CrossingMAParams>
    ) -> HttpResponse {
    get_ma_signal(symbol, "SMA", query).await
}

pub async fn get_ewma_signal(
    symbol: web::Path<String>, 
    query:Query<CrossingMAParams>
    ) -> HttpResponse {
    get_ma_signal(symbol, "EWMA", query).await
}