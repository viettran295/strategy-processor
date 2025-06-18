use actix_web::{web::{self, Query}, HttpResponse};
use log::error;
use serde::Deserialize;

use crate::{fetch::StockFetcher, scanner::{ScannerCrossingMA, ScannerPerformance, ScannerRSI}};
use crate::processor::{Strategy, StrategyCrossingMA, StrategyRSI, StrategyBollingerBands};
use crate::converter::DfConverter;
use crate::db::DbManager;

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
        symbol.clone(), 
        &query, 
        |df_proc, _| {
            let ma_window = 14;
            let upper_bound = 80;
            let lower_bound = 20;
            let mut rsi_str = StrategyRSI::new(df_proc.df.unwrap(), ma_window, upper_bound, lower_bound);
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

pub async fn get_best_performance_rsi(
    symbol: web::Path<String>, 
    query: Query<QueryParams>,
) -> HttpResponse {
    fetch_and_process(
        symbol.clone(), 
        &query,
        |df_proc, _| {
            let from_ma = 5;
            let to_ma = 20;
            let ma_window = 5;
            let upper_bound = 80;
            let lower_bound = 20;
            let rsi_str = StrategyRSI::new(df_proc.df.unwrap(), ma_window, upper_bound, lower_bound);
            let mut scanner = ScannerRSI::new(rsi_str.clone(), from_ma, to_ma);
            match scanner.get_best_performance_df() {
                Some(df) => {
                    let response = DfConverter::rsi_df_to_json(&df);
                    Ok(response)
                },
                None => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No RSI best performance found"
                )))
            }
        }
    ).await
}

pub async fn get_bb_signal(
    symbol: web::Path<String>,
    query: Query<QueryParams>
) -> HttpResponse {
    fetch_and_process(
        symbol.clone(), 
        &query, 
        |df_proc, _| {
            let mut bb = StrategyBollingerBands::new(
                                        df_proc.df.unwrap(), 
                                        20
                                );
            match bb.calc_signal() {
                Ok(_) => {
                    let response = DfConverter::bb_df_to_json(&bb.df.clone().unwrap());
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

async fn fetch_and_process<F>(
    symbol: String,
    query: &Query<QueryParams>,
    process_fn: F
) -> HttpResponse
where F: FnOnce(DfConverter, &Query<QueryParams>) -> Result<String, Box<dyn std::error::Error>> 
{
    let mut df_cvt = DfConverter::new();
    let db = DbManager::default();

    let df = if db.table_exists(symbol.to_string()).unwrap_or(false){
        df_cvt.df = Some(db.get_table(symbol.to_string()).unwrap());
        df_cvt
    } else {
        let fetcher = StockFetcher::new();
        let start_date = query.start_date.clone();
        let end_date = query.end_date.clone();
        fetcher.fetch_prices(symbol.to_string(), start_date, end_date)
            .await
            .and_then(|stock_data| {
                df_cvt.to_df(stock_data)?;
                db.create_table(symbol.to_string(), &mut df_cvt.df.clone().unwrap())
                    .expect(format!("Error creating {} db table", symbol).as_str());
                Ok(df_cvt)
            }).expect("Error fetching stock price")
    };

    match process_fn(df, query) {
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
}

async fn get_ma_signal(
    symbol: web::Path<String>,
    ma_type: &str,
    query: Query<QueryParams>
) -> HttpResponse {
    let ma_type_owned = ma_type.to_string();
    fetch_and_process(
        symbol.clone(), 
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
        symbol.clone(), 
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
            match scanner.get_best_performance_df() {
                Some(df) => {
                    let response = DfConverter::crossingma_df_to_json(&df);
                    Ok(response)
                },
                None => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No crossing MA best performance found"
                )))
            }
        }
    ).await
}

