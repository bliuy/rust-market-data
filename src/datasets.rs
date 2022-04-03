//! Objective: The main purpose of this module is to provide connectivity to various data sources to pull data from.
use super::enums;
use super::errors;
use super::parsers;
use super::requests;
use chrono;
use serde;
use serde::Deserialize;

struct TickerInfo<'a> {
    ticker_symbol: &'a str,
    start_date: chrono::DateTime<chrono::Utc>,
    end_date: chrono::DateTime<chrono::Utc>,
}

struct RawPrice<'a> {
    ticker: &'a str,
    open_prices: &'a [f64],
    close_prices: &'a [f64],
    high_prices: &'a [f64],
    low_prices: &'a [f64],
    volume: &'a [i64],
    currency: enums::Currency,
    adj_close: Option<&'a [f64]>,
}

fn source_yahoo_finance<'a>(
    ticker_info: &'a TickerInfo,
) -> Result<Vec<RawPrice<'a>>, errors::SourceDataError> {
    const BASE_URL: &str = "https://query1.finance.yahoo.com/v7/finance/download";
    let start_date = ticker_info.start_date;
    let end_date = ticker_info.end_date;
    let ticker = ticker_info.ticker_symbol;

    // Defining the record struct for parsing the response content
    #[derive(Debug, serde::Deserialize)]
    struct Record {
        // #[serde(rename = "Date")]
        // timestamp:Option<chrono::DateTime<chrono::Utc>>,
        #[serde(rename = "Open")]
        open_price: Option<f32>,
        close_price: Option<f32>,
        high_price: Option<f32>,
        low_price: Option<f32>,
        adj_close_price: Option<f32>,
        volume: Option<f32>,
    }

    // Constructing the URL
    let url = format!(
        "{}/{}?period1={}&period2={}&interval=1d&events=history&includeAdjustedClose=true",
        BASE_URL, ticker, start_date, end_date
    );

    // Sending the GET request
    let response = requests::blocking_reqwest(&url)?;

    // Parsing the raw data
    let response_bytes = parsers::parse_blocking_response_bytes(response)?;

    //

    todo!()
}
