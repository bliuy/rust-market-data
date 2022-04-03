//! Objective: The main purpose of this module is to provide connectivity to various data sources to pull data from.
use std::sync::mpsc::Receiver;

use super::enums;
use super::errors;
use super::parsers;
use super::requests;
use csv;

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
        #[serde(rename = "Date")]
        #[serde(deserialize_with = "parsers::serde_parsers::parsing_std_dates")]
        // Note that csv::invalid_option is not used here, since the functionality is already provided by the custom deserializer.
        timestamp: Option<chrono::DateTime<chrono::Utc>>,
        #[serde(rename = "Open")]
        #[serde(deserialize_with = "csv::invalid_option")]
        open_price: Option<f32>,
        #[serde(rename = "Close")]
        #[serde(deserialize_with = "csv::invalid_option")]
        close_price: Option<f32>,
        #[serde(rename = "High")]
        #[serde(deserialize_with = "csv::invalid_option")]
        high_price: Option<f32>,
        #[serde(rename = "Low")]
        #[serde(deserialize_with = "csv::invalid_option")]
        low_price: Option<f32>,
        #[serde(rename = "Adj Close")]
        #[serde(deserialize_with = "csv::invalid_option")]
        adj_close_price: Option<f32>,
        #[serde(rename = "Volume")]
        #[serde(deserialize_with = "csv::invalid_option")]
        volume: Option<f32>,
    }

    // Constructing the URL
    let url = format!(
        "{}/{}?period1={}&period2={}&interval=1d&events=history&includeAdjustedClose=true",
        BASE_URL, ticker, start_date, end_date
    );

    // Sending the GET request
    let response = requests::blocking_reqwest(&url)?;

    // Parsing the raw data into a bytes array
    let response_bytes = parsers::parse_blocking_response_bytes(response)?;

    // Parsing the response bytes array into a csv reader
    let mut csv_reader = csv::ReaderBuilder::new().from_reader(&*response_bytes);

    let records = csv_reader
        .into_deserialize()
        .filter_map(|raw_record| match raw_record {
            Ok(rec) => {
                let record: Record = rec;
                Some(record)
            }
            Err(e) => None,
        })
        .collect::<Vec<Record>>();

    todo!()
}
