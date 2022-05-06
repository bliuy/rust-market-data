//! Objective: The main purpose of this module is to provide connectivity to various data sources to pull data from.
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::enums;
use super::errors;
use super::parsers;
use super::requests;

use chrono::TimeZone;

pub mod structs {
    use super::*;

    #[derive(Debug)]
    pub struct YahooFinancePriceRecord {
        pub(super) ticker: String,
        pub(super) timestamps: Vec<chrono::DateTime<chrono::Utc>>,
        pub(super) open_prices: Vec<f32>,
        pub(super) close_prices: Vec<f32>,
        pub(super) high_prices: Vec<f32>,
        pub(super) low_prices: Vec<f32>,
        pub(super) volume: Vec<i32>,
        pub(super) currency: enums::Currency,
        pub(super) adj_close: Vec<f32>,
    }

    impl YahooFinancePriceRecord {
        pub fn new(ticker_symbol: &str, num_of_records: usize, currency: enums::Currency) -> Self {
            YahooFinancePriceRecord {
                ticker: ticker_symbol.to_owned(),
                timestamps: Vec::with_capacity(num_of_records),
                open_prices: Vec::with_capacity(num_of_records),
                close_prices: Vec::with_capacity(num_of_records),
                high_prices: Vec::with_capacity(num_of_records),
                low_prices: Vec::with_capacity(num_of_records),
                volume: Vec::with_capacity(num_of_records),
                currency,
                adj_close: Vec::with_capacity(num_of_records),
            }
        }

        pub fn get_adj_close_prices(&self) -> &[f32] {
            &self.adj_close
        }
    }

    impl traits::Prices for YahooFinancePriceRecord {
        fn get_high_prices(&self) -> &[f32] {
            &self.high_prices
        }

        fn get_low_prices(&self) -> &[f32] {
            &self.low_prices
        }

        fn get_open_prices(&self) -> &[f32] {
            &self.open_prices
        }

        fn get_close_prices(&self) -> &[f32] {
            &self.close_prices
        }

        fn get_currency(&self) -> enums::Currency {
            self.currency
        }
    }

    impl traits::Timestamps for YahooFinancePriceRecord {
        fn get_timestamps(&self) -> &[chrono::DateTime<chrono::Utc>] {
            &self.timestamps
        }
    }

    impl traits::Volume for YahooFinancePriceRecord {
        fn get_volume(&self) -> &[i32] {
            &self.volume
        }
    }

    impl traits::Description for YahooFinancePriceRecord {
        fn get_ticker_symbol(&self) -> &str {
            &self.ticker
        }
    }

    impl traits::PriceDeltas for YahooFinancePriceRecord {}

    impl traits::PriceDeltaPercentages for YahooFinancePriceRecord {}

    #[derive(PartialEq)]
    pub struct TickerInfo<'a> {
        pub(super) ticker_symbol: &'a str,
        pub(super) start_datetime: chrono::DateTime<chrono::Utc>,
        pub(super) end_datetime: chrono::DateTime<chrono::Utc>,
        pub(super) currency: enums::Currency,
    }

    impl<'a> TickerInfo<'a> {
        pub fn new(
            ticker_symbol: &'a str, // Guarantees that the reference to the char array (&str) lives as long as the TickerInfo object.
            start_timestamp: &str,
            end_timestamp: &str,
            currency: enums::Currency,
        ) -> Result<Self, errors::InitializationError> {
            const DATE_FORMAT: &str = "%Y-%m-%d  %H:%M:%S";

            let start_datetime_input = format!("{} 00:00:00", start_timestamp);
            let end_datetime_input = format!("{} 00:00:00", end_timestamp);

            

            // Processing the start and end dates into chrono datetime objects
            let start_datetime = match chrono::Utc.datetime_from_str(&start_datetime_input, DATE_FORMAT) {
                Ok(i) => i,
                Err(e) => {
                    return Err(errors::InitializationError::TickerInfoInitializationError(
                        format!("{:#?}", e), // Returning the printed output of the internal error.
                    ));
                }
            };
            let end_datetime = match chrono::Utc.datetime_from_str(&end_datetime_input, DATE_FORMAT) {
                Ok(i) => i,
                Err(e) => {
                    return Err(errors::InitializationError::TickerInfoInitializationError(
                        format!("{:#?}", e), // Returning the printed output of the internal error.
                    ));
                }
            };

            // Constructing the TickerInfo Object
            Ok(TickerInfo {
                ticker_symbol,
                start_datetime,
                end_datetime,
                currency,
            })
        }
    }
}

pub mod traits {

    use super::*;

    pub trait Timestamps {
        fn get_timestamps(&self) -> &[chrono::DateTime<chrono::Utc>];
    }

    /// The Prices trait provides a set of functions for retrieval of pricing data in the form of arrays.
    pub trait Prices {
        fn get_high_prices(&self) -> &[f32];

        fn get_low_prices(&self) -> &[f32];

        fn get_open_prices(&self) -> &[f32];

        fn get_close_prices(&self) -> &[f32];

        fn get_currency(&self) -> enums::Currency;
    }

    pub trait Volume {
        fn get_volume(&self) -> &[i32];
    }

    pub trait Description {
        fn get_ticker_symbol(&self) -> &str;
    }

    pub trait PriceDeltas: Prices {
        fn get_high_prevclose_pricedelta(&self) -> Vec<f32> {
            let high_prices = self.get_high_prices();
            let close_prices = self.get_close_prices();
            let num_of_records = high_prices.len();
            let mut result: Vec<f32> = vec![f32::NAN];

            high_prices[1..]
                .iter()
                .zip(close_prices[..num_of_records - 1].iter())
                .for_each(|(high, close)| result.push(high - close));
            result
        }

        fn get_prevclose_low_pricedelta(&self) -> Vec<f32> {
            let low_prices = self.get_low_prices();
            let close_prices = self.get_close_prices();
            let num_of_records = low_prices.len();
            let mut result: Vec<f32> = vec![f32::NAN];

            low_prices[1..]
                .iter()
                .zip(close_prices[..num_of_records - 1].iter())
                .for_each(|(low, close)| result.push(close - low));
            result
        }
    }

    pub trait PriceDeltaPercentages: PriceDeltas {
        fn get_high_prevclose_pricedelta_percentage(&self) -> Vec<f32> {
            let high_prevclose_pricedelta = self.get_high_prevclose_pricedelta();
            let close_prices = self.get_close_prices();
            let num_of_records = high_prevclose_pricedelta.len();
            let mut result: Vec<f32> = Vec::new();

            high_prevclose_pricedelta
                .into_iter()
                .zip(close_prices[..num_of_records].iter())
                .for_each(|(high_prevclose, close)| result.push((high_prevclose / close) * 100.0));
            result
        }

        fn get_prevclose_low_pricedelta_percentage(&self) -> Vec<f32> {
            let prevclose_low_pricedelta = self.get_prevclose_low_pricedelta();
            let close_prices = self.get_close_prices();
            let num_of_records = prevclose_low_pricedelta.len();
            let mut result: Vec<f32> = Vec::new();

            prevclose_low_pricedelta
                .into_iter()
                .zip(close_prices[..num_of_records].iter())
                .for_each(|(high_prevclose, close)| result.push((high_prevclose / close) * 100.0));
            result
        }
    }
}

pub fn source_yahoo_finance<'a>(
    ticker_info: &'a structs::TickerInfo,
) -> Result<structs::YahooFinancePriceRecord, errors::SourceDataError> {
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
        volume: Option<i32>,
    }

    impl PartialEq for Record {
        fn eq(&self, other: &Self) -> bool {
            let x = self.timestamp.expect("Should be unreachable since all invalid timestamps were filtered out during the deserializing process.");
            let y = other.timestamp.expect("Should be unreachable since all invalid timestamps were filtered out during the deserializing process.");
            x == y
        }
    }

    impl PartialOrd for Record {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            let x = self.timestamp.expect("Should be unreachable since all invalid timestamps were filtered out during the deserializing process.");
            let y = other.timestamp.expect("Should be unreachable since all invalid timestamps were filtered out during the deserializing process.");
            Some(x.cmp(&y))
        }
    }

    impl Eq for Record {}

    impl Ord for Record {
        fn cmp(&self, other: &Self) -> Ordering {
            let x = self.timestamp.expect("Should be unreachable since all invalid timestamps were filtered out during the deserializing process.");
            let y = other.timestamp.expect("Should be unreachable since all invalid timestamps were filtered out during the deserializing process.");
            x.cmp(&y)
        }
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
                record.timestamp?;
                Some(record) // Only these records will be stored in the records vec
            }
            Err(_e) => None, // These records will be filtered out
        })
        .collect::<BinaryHeap<Record>>() // Sorted by Max heap - Latest Timestamp first
        .into_sorted_vec(); // Parsing into a sorted vec - into_iter_sorted method only available on nightly

    // Instantiating the YahooFinancePriceRecord Struct
    let records_count = records.len();
    let mut price_record = structs::YahooFinancePriceRecord::new(
        // Mutable to allow for additional of elements to its internal Vecs
        ticker_info.ticker_symbol,
        records_count,
        ticker_info.currency, // Copy occurs here instead of a move, since the Copy trait was derived for a simple enum
    ); // Allocating the Vec capacity upfront to avoid re-allocation as the Vec grows when the records are loaded into it.

    // Loading the records into the struct
    for record in records.into_iter() {
        if let Some(i) = record.timestamp {
            price_record.timestamps.push(i)
        } else {
            return Err(errors::SourceDataError::MissingDataError(
                "Missing value identified in the timestamp field.".to_string(),
            ));
        }

        if let Some(i) = record.open_price {
            price_record.open_prices.push(i)
        } else {
            return Err(errors::SourceDataError::MissingDataError(
                "Missing value identified in the open price field.".to_string(),
            ));
        }

        if let Some(i) = record.close_price {
            price_record.close_prices.push(i)
        } else {
            return Err(errors::SourceDataError::MissingDataError(
                "Missing value identified in the close price field.".to_string(),
            ));
        }

        if let Some(i) = record.high_price {
            price_record.high_prices.push(i)
        } else {
            return Err(errors::SourceDataError::MissingDataError(
                "Missing value identified in the high price field.".to_string(),
            ));
        }

        if let Some(i) = record.low_price {
            price_record.low_prices.push(i)
        } else {
            return Err(errors::SourceDataError::MissingDataError(
                "Missing value identified in the low price field.".to_string(),
            ));
        }

        if let Some(i) = record.adj_close_price {
            price_record.adj_close.push(i)
        } else {
            return Err(errors::SourceDataError::MissingDataError(
                "Missing value identified in the adj close field.".to_string(),
            ));
        }

        if let Some(i) = record.volume {
            price_record.volume.push(i)
        } else {
            return Err(errors::SourceDataError::MissingDataError(
                "Missing value identified in the volume field.".to_string(),
            ));
        }
    }

    Ok(price_record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datasets::traits::*;

    #[test]
    fn ticker_info_struct_new() {
        let foo = structs::TickerInfo::new(
            "AAPL",
            "2020-01-01 00:00:00",
            "2020-01-03 00:00:00",
            enums::Currency::Usd,
        )
        .unwrap(); // Unwrapped used, panics will cause the test to fail.
        let bar = structs::TickerInfo {
            ticker_symbol: "AAPL",
            start_datetime: chrono::Utc.ymd(2020, 1, 1).and_hms(0, 0, 0),
            end_datetime: chrono::Utc.ymd(2020, 1, 3).and_hms(0, 0, 0),
            currency: enums::Currency::Usd,
        };
        assert!(foo == bar)
    }

    #[test]
    fn test_source_yahoo_finance() {
        let foo = structs::TickerInfo::new(
            "AAPL",
            "2022-01-01 00:00:00",
            "2022-01-05 00:00:00",
            enums::Currency::Usd,
        )
        .unwrap(); // Unwrapped used, panics will cause the test to fail.
        let bar = source_yahoo_finance(&foo).unwrap(); // Result from the function

        // Checking the individual attributes
        let test_case_timestamps: Vec<chrono::DateTime<chrono::Utc>> = vec![
            chrono::Date::from_utc(chrono::NaiveDate::from_ymd(2022, 1, 3), chrono::Utc)
                .and_hms(0, 0, 0),
            chrono::Date::from_utc(chrono::NaiveDate::from_ymd(2022, 1, 4), chrono::Utc)
                .and_hms(0, 0, 0),
        ];
        assert!(bar.get_timestamps() == test_case_timestamps);

        let test_case_open: Vec<f32> = vec![177.83, 182.63];
        assert!(bar.get_open_prices() == test_case_open);

        let test_case_high: Vec<f32> = vec![182.88, 182.94];
        assert!(bar.get_high_prices() == test_case_high);

        let test_case_low: Vec<f32> = vec![177.71, 179.12];
        assert!(bar.get_low_prices() == test_case_low);

        let test_case_close: Vec<f32> = vec![182.01, 179.70];
        assert!(bar.get_close_prices() == test_case_close);

        let test_case_adjclose: Vec<f32> = vec![181.7784, 179.47134];
        assert!(bar.get_adj_close_prices() == test_case_adjclose);

        let test_case_volume: Vec<i32> = vec![104487900, 99310400];
        assert!(bar.get_volume() == test_case_volume);
    }
}
