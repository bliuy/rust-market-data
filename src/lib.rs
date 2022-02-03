use std::string::ParseError;

use chrono::TimeZone;

pub enum Currency {
    USD(f64),
    SGD(f64),
}

pub enum Data_Provider {
    Yahoo_Finance,
}

pub struct HistoricalPrices {
    ticker_symbol: String,
    timestamp: Vec<chrono::DateTime<chrono::Utc>>,
    open_price: Vec<Currency>,
    high_price: Vec<Currency>,
    low_price: Vec<Currency>,
    close_price: Vec<Currency>,
}

impl HistoricalPrices {
    fn _load_from_datasource(
        ticker_symbol: &str,
        start_datetime: chrono::DateTime<chrono::Utc>,
        end_datetime: chrono::DateTime<chrono::Utc>,
    ) {
        let base_url = "https://query1.finance.yahoo.com/v7/finance/download/XLK?";
        let period1 = start_datetime.timestamp();
        let period2 = end_datetime.timestamp();

        // Constructing the complete url string to make the request
        let url = format!(
            "{}?period1={}&period2={}&interval=1d&events=history&includeAdjustedClose=true",
            base_url, period1, period2
        );

        // Sending the GET request
        let response_bytes = reqwest::blocking::get(url)
            .unwrap()
            .bytes()
            .unwrap()
            .into_iter()
            .collect::<Vec<u8>>();

        // Parsing the response into a csv file
        let rdr = csv::ReaderBuilder::new().from_reader(&response_bytes[..]);
    }

    fn new(
        ticker_symbol: String,
        start_datetime: chrono::DateTime<chrono::Utc>,
        end_datetime: chrono::DateTime<chrono::Utc>,
    ) {
    }
}

pub fn chrono_strptime(
    timestamp: &str,
) -> std::result::Result<chrono::DateTime<chrono::Utc>, Box<dyn std::error::Error>> {
    /// Function takes a string datetime formatted as "YYYY-MM-DD" and converts it to a chrono::Utc object.
    let datetime_array_result: std::result::Result<Vec<_>, _> =
        timestamp.split("-").map(|x| x.parse::<u32>()).collect();

    let datetime_array = match datetime_array_result {
        Ok(i) => i,
        Err(e) => return Err(Box::new(e)),
    };

    let result = chrono::Utc
        .ymd(
            i32::try_from(datetime_array[0])?,
            datetime_array[1],
            datetime_array[2],
        )
        .and_hms(0, 0, 0);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::{num::ParseIntError, string::ParseError};

    use super::*;

    #[test]
    fn test_chrono_strptime() {
        let result = chrono_strptime("2022-02-01").unwrap();
        assert_eq!(result, chrono::Utc.ymd(2022, 2, 1).and_hms(0, 0, 0))
    }

    #[test]
    fn test_load_from_datasource() {
        HistoricalPrices::_load_from_datasource(
            "XLK",
            chrono::Utc.ymd(2022, 1, 1).and_hms(0, 0, 0),
            chrono::Utc.ymd(2022, 2, 1).and_hms(0, 0, 0),
        )
    }

    // #[test]
    // fn test_chrono_strptime_error_handling() {
    //     let result = match chrono_strptime("2022-2-A") {
    //         Ok(result) => result,
    //         Err(e) => {
    //             if let Ok(e) = e.downcast::<ParseIntError>() {
    //                 println!("Provide correct datetime formatting!");
    //                 panic!("Please ensure correct datetime format is provided!");
    //             } else {
    //                 panic!("Unknown error occured.");
    //             }
    //         }
    // };
    // }
}
