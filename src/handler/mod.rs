use actix_web::{web::{self, Query}, HttpResponse};
use log::error;
use serde::Deserialize;

use crate::{fetch::StockFetcher, scanner::{ScannerPerformance, ScannerCrossingMA}};
use crate::processor::{Strategy, StrategyCrossingMA, StrategyRSI};
use crate::converter::DfConverter;

#[derive(Deserialize)]
pub struct DateParams {
    start_date: Option<String>,
    end_date: Option<String>,
}

#[derive(Deserialize)]
pub struct QueryParams {
    start_date: Option<String>,
    end_date: Option<String>,
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

pub async fn get_sma_signal(
    symbol: web::Path<String>, 
    query:Query<QueryParams>
    ) -> HttpResponse {
    get_ma_signal(symbol, "SMA", query).await
}

pub async fn get_ewma_signal(
    symbol: web::Path<String>, 
    query:Query<QueryParams>
    ) -> HttpResponse {
    get_ma_signal(symbol, "EWMA", query).await
}

pub async fn get_rsi_signal(
    symbol: web::Path<String>,
    query: Query<QueryParams>
    ) -> HttpResponse {
    fetch_and_process(
        symbol, 
        &query, 
        |df_proc, _| {
            let mut rsi_str = StrategyRSI::new(df_proc.df.unwrap(), 14, 80, 20);
            match rsi_str.calc_rsi() {
                Ok(_) => {
                    rsi_str.calc_signal()?;
                    let response = DfConverter::rsi_df_to_json(&rsi_str.df.clone().unwrap());
                    return Ok(response);
                }
                Err(e) => {
                    error!("Error calculating RSI stock prices: {}", e);
                    return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Error calculating signal: {}", e)
                            )));
                }
            }
        }
    ).await
}

pub async fn get_best_performance_sma(
    symbol: web::Path<String>, 
    query:Query<QueryParams>
    ) -> HttpResponse {
    get_best_performance_ma(symbol, query, "SMA").await
}

pub async fn get_best_performance_ewma(
    symbol: web::Path<String>, 
    query:Query<QueryParams>
    ) -> HttpResponse {
    get_best_performance_ma(symbol, query, "EWMA").await
}

async fn fetch_and_process<F>(
    symbol: web::Path<String>,
    query: &Query<QueryParams>,
    process_fn: F
) -> HttpResponse
where F: FnOnce(DfConverter, &Query<QueryParams>) -> Result<String, Box<dyn std::error::Error>> 
{
    let fetcher = StockFetcher::new();
    let start_date = query.start_date.clone();
    let end_date = query.end_date.clone();

    match fetcher.fetch_prices(symbol.to_string(), start_date, end_date).await {
        Ok(stock_data) => {
            let mut df_proc = DfConverter::new();
            match df_proc.to_df(stock_data) {
                Ok(_) => {
                    match process_fn(df_proc, query) {
                        Ok(response) => {
                            return HttpResponse::Ok()
                                .content_type("application/json")
                                .body(response);
                        },
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
            return HttpResponse::InternalServerError()
                .body(format!("Error getting moving average stock prices: {}", e))
        }
    }
}

async fn get_ma_signal(
    symbol: web::Path<String>,
    ma_type: &str,
    query: Query<QueryParams>
) -> HttpResponse {
    let ma_type_owned = ma_type.to_string();
    fetch_and_process(
        symbol, 
        &query, 
        |df_proc, query| {
            let short_ma = query.short_ma.clone().unwrap_or(20);
            let long_ma = query.long_ma.clone().unwrap_or(50);
            let mut crs_avg = StrategyCrossingMA::new(
                                        df_proc.df.unwrap(), 
                                        short_ma, 
                                        long_ma, 
                                        ma_type_owned
                                );
            match crs_avg.calc_signal() {
                Ok(_) => {
                    let response = DfConverter::crossingma_df_to_json(&crs_avg.df.clone().unwrap());
                    Ok(response)
                }
                Err(e) => {
                    error!("Error calculating signal: {}", e);
                    Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Error calculating signal: {}", e)
                    )))
                }
            }
        }
    ).await
}

async fn get_best_performance_ma(
    symbol: web::Path<String>, 
    query: Query<QueryParams>,
    ma_type: &str
) -> HttpResponse {
    let ma_type_owned = ma_type.to_string();
    fetch_and_process(
        symbol, 
        &query,
        |df_proc, query| {
            let from_ma = 10;
            let to_ma = 200;
            let short_ma = query.short_ma.clone().unwrap_or(20);
            let long_ma = query.long_ma.clone().unwrap_or(50);
            let crs_avg = StrategyCrossingMA::new(
                                    df_proc.df.unwrap(), 
                                    short_ma, 
                                    long_ma, 
                                    ma_type_owned
                            );
            let mut scanner = ScannerCrossingMA::new(crs_avg.clone(), from_ma, to_ma);
            scanner.scan_performance()?;
            match scanner.best_performance() {
                Some((strategy, value)) => {
                    let response = serde_json::json!({
                        "strategy": strategy,
                        "value": value
                    }).to_string();
                    Ok(response)
                },
                None => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No best performance found"
                )))
            }
        }
    ).await
}