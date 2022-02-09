use serde::{Deserialize, Serialize};
use std::string::ParseError;

use chrono::TimeZone;

#[derive(Debug, Serialize, Deserialize)]
pub enum Currency {
    USD,
    SGD,
}

pub enum DataProvider {
    YahooFinance,
}

mod csv_datetime_formatting {
    use chrono::TimeZone;
    use serde::Deserialize;

    // use super::*;
    static SERIALIZE_DATETIME_FORMAT_STRING: &str = "%Y-%m-%d";
    static DESERIALIZE_DATETIME_FORMAT_STRING: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(
        date: &chrono::DateTime<chrono::Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!("{}", date.format(SERIALIZE_DATETIME_FORMAT_STRING));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut s = String::deserialize(deserializer)?;
        s.push_str(" 00:00:00");
        let result = chrono::Utc
            .datetime_from_str(&s, DESERIALIZE_DATETIME_FORMAT_STRING)
            .map_err(serde::de::Error::custom);
        result
    }
}

#[derive(Debug)]
pub struct PriceRecord {
    ticker_symbol: String,
    timestamp: Vec<chrono::DateTime<chrono::Utc>>,
    open_price: Vec<f64>,
    high_price: Vec<f64>,
    low_price: Vec<f64>,
    close_price: Vec<f64>,
    adj_close: Vec<f64>,
    volume: Vec<i64>,
    currency: Currency,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CsvRecord {
    #[serde(rename = "Date")]
    #[serde(with = "csv_datetime_formatting")]
    timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "Open")]
    open_price: f64,
    #[serde(rename = "High")]
    high_price: f64,
    #[serde(rename = "Low")]
    low_price: f64,
    #[serde(rename = "Close")]
    close_price: f64,
    #[serde(rename = "Adj Close")]
    adj_close: f64,
    #[serde(rename = "Volume")]
    volume: i64,
}

pub fn get_prices(
    ticker_symbol: &str,
    start_datetime: chrono::DateTime<chrono::Utc>,
    end_datetime: chrono::DateTime<chrono::Utc>,
) -> Result<(PriceRecord), Box<dyn std::error::Error>> {
    let base_url = "https://query1.finance.yahoo.com/v7/finance/download/XLK?";
    let period1 = start_datetime.timestamp();
    let period2 = end_datetime.timestamp();
    let mut csv_vec: Vec<CsvRecord> = Vec::new();

    // Constructing the complete url string to make the request
    let url = format!(
        "{}?period1={}&period2={}&interval=1d&events=history&includeAdjustedClose=true",
        base_url, period1, period2
    );
    println!("{}",url);

    // Sending the GET request
    let response_bytes = reqwest::blocking::get(url)
        .expect("GET request failed!")
        .bytes()
        .unwrap()
        .into_iter()
        .collect::<Vec<u8>>(); // Collecting into a byte array

    // Parsing the response into a csv file
    let mut rdr = csv::ReaderBuilder::new().from_reader(&response_bytes[..]);

    // Deserializing the csv file
    for result in rdr.deserialize() {
        let csv_record: CsvRecord = result?;
        csv_vec.push(csv_record);
    }

    // Sorting the vec of CsvRecords
    csv_vec.sort_by(|x, y| x.timestamp.cmp(&y.timestamp));

    // Constructing the PriceRecord Object
    let price_record = PriceRecord {
        ticker_symbol: String::from(ticker_symbol),
        timestamp: csv_vec
            .iter()
            .map(|x| x.timestamp)
            .collect::<Vec<chrono::DateTime<chrono::Utc>>>(),
        open_price: csv_vec.iter().map(|x| x.open_price).collect::<Vec<f64>>(),
        high_price: csv_vec.iter().map(|x| x.high_price).collect::<Vec<f64>>(),
        low_price: csv_vec.iter().map(|x| x.low_price).collect::<Vec<f64>>(),
        close_price: csv_vec.iter().map(|x| x.close_price).collect::<Vec<f64>>(),
        adj_close: csv_vec.iter().map(|x| x.adj_close).collect::<Vec<f64>>(),
        volume: csv_vec.iter().map(|x| x.volume).collect::<Vec<i64>>(),
        currency: Currency::USD,
    };
    Ok((price_record))
}

pub fn chrono_strptime(
    timestamp: &str,
) -> std::result::Result<chrono::DateTime<chrono::Utc>, Box<dyn std::error::Error>> {
    // Function takes a string datetime formatted as "YYYY-MM-DD" and converts it to a chrono::Utc object.
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

pub fn groupby_closing_price(price_record: PriceRecord, bin_timestamp: chrono::Duration) -> Option<()> {
    let starting_time = match price_record.timestamp.first() {
        Some(i) => i,
        None => return None,
    };
    let ending_time = match price_record.timestamp.last() {
        Some(i) => i,
        None => return None,
    };

    let mut bins: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    let mut prev_dt = *starting_time;
    let mut next_dt = prev_dt + bin_timestamp;
    let mut groups: std::collections::HashMap<
        chrono::DateTime<chrono::Utc>,
        Vec<f64>,
    > = std::collections::HashMap::new();
    let mut i = 0;

    while true {
        let mut group:Vec<f64>=Vec::new();
        while price_record.timestamp[i] < next_dt {
            group.push(price_record.close_price[i]);
            i+=1;
            if i >= price_record.timestamp.len() {
                break
            }
        }
        groups.insert(prev_dt, group);
        prev_dt = next_dt;
        next_dt = next_dt + bin_timestamp;
        if next_dt > *ending_time {
            break
        }
    }

    println!("{:?}",groups);

    Some(())
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
    fn test_get_prices() {
        match get_prices(
            "XLK",
            chrono::Utc.ymd(2022, 1, 1).and_hms(0, 0, 0),
            chrono::Utc.ymd(2022, 2, 1).and_hms(0, 0, 0),
        ) {
            Ok(i) => println!("OK"),
            Err(e) => println!("{}", e),
        };
        ()
    }

    #[test]
    fn test_groupby() {
        let price_record = get_prices(
            "XLK",
            chrono::Utc.ymd(2022, 1, 1).and_hms(0, 0, 0),
            chrono::Utc.ymd(2022, 2, 1).and_hms(0, 0, 0),
        )
        .unwrap();

        groupby_closing_price(price_record, chrono::Duration::weeks(1));
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
