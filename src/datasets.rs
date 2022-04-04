//! Objective: The main purpose of this module is to provide connectivity to various data sources to pull data from.
use super::enums;
use super::errors;
use super::parsers;
use super::requests;
use chrono;
use chrono::TimeZone;

#[derive(PartialEq)]
struct TickerInfo<'a> {
    ticker_symbol: &'a str,
    start_datetime: chrono::DateTime<chrono::Utc>,
    end_datetime: chrono::DateTime<chrono::Utc>,
}

impl<'a> TickerInfo<'a> {
    fn new(
        ticker_symbol: &'a str, // Guarantees that the reference to the char array (&str) lives as long as the TickerInfo object.
        start_timestamp: &str,
        end_timestamp: &str,
    ) -> Result<Self, errors::InitializationError> {

        const DATE_FORMAT: &str = "%Y-%m-%d  %H:%M:%S";

        // Processing the start and end dates into chrono datetime objects
        let start_datetime = match chrono::Utc.datetime_from_str(start_timestamp, DATE_FORMAT) {
            Ok(i) => i,
            Err(e) => {
                return Err(errors::InitializationError::TickerInfoInitializationError(
                    String::from(format!("{:#?}", e)), // Returning the printed output of the internal error.
                ))
            }
        };
        let end_datetime = match chrono::Utc.datetime_from_str(end_timestamp, DATE_FORMAT) {
            Ok(i) => i,
            Err(e) => {
                return Err(errors::InitializationError::TickerInfoInitializationError(
                    String::from(format!("{:#?}", e)), // Returning the printed output of the internal error.
                ))
            }
        };

        // Constructing the TickerInfo Object
        Ok(TickerInfo{
            ticker_symbol,
            start_datetime,
            end_datetime, 
        })
    }
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
    let start_date = ticker_info.start_datetime.timestamp();
    let end_date = ticker_info.end_datetime.timestamp();
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
    let csv_reader = csv::ReaderBuilder::new().from_reader(&*response_bytes);

    let records = csv_reader
        .into_deserialize()
        .filter_map(|raw_record| match raw_record {
            Ok(rec) => {
                let record: Record = rec;
                Some(record)
            }
            Err(_e) => None,
        })
        .collect::<Vec<Record>>();

    println!("{:#?}", records);


    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticker_info_struct_new() {
        let foo = TickerInfo::new("AAPL", "2020-01-01 00:00:00", "2020-01-03 00:00:00").unwrap(); // Unwrapped used, panics will cause the test to fail.
        let bar = TickerInfo{
            ticker_symbol: "AAPL",
            start_datetime: chrono::Utc.ymd(2020, 1, 1).and_hms(0,0,0),
            end_datetime: chrono::Utc.ymd(2020, 1, 3).and_hms(0,0,0),
        };
        assert!(foo==bar)
    }

    #[test]
    fn test_source_yahoo_finance() {
        let foo = TickerInfo::new("AAPL", "2020-01-01 00:00:00", "2020-01-07 00:00:00").unwrap(); // Unwrapped used, panics will cause the test to fail.
        source_yahoo_finance(&foo);
    }
    
}
