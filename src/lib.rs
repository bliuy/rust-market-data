use serde::{Deserialize, Serialize};
use std::{ops::Range, string::ParseError};

use chrono::{Datelike, TimeZone};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug)]
pub struct GroupedPriceRecord {
    ticker_symbol: String,
    groupby_duration: chrono::Duration,
    binned_timestamps: Vec<chrono::DateTime<chrono::Utc>>,
    grouping_indexes: Vec<std::ops::Range<usize>>,
    open_price: Vec<Vec<f64>>,
    high_price: Vec<Vec<f64>>,
    low_price: Vec<Vec<f64>>,
    close_price: Vec<Vec<f64>>,
    adj_close: Vec<Vec<f64>>,
    volume: Vec<Vec<i64>>,
    open_low_pricedelta: Option<Vec<Vec<f64>>>,
    open_high_pricedelta: Option<Vec<Vec<f64>>>,
    prevclose_low_pricedelta: Option<Vec<Vec<f64>>>,
    prevclose_high_pricedelta: Option<Vec<Vec<f64>>>,
    currency: Currency,
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
        "{}period1={}&period2={}&interval=1d&events=history&includeAdjustedClose=true",
        base_url, period1, period2
    );

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

impl PriceRecord {
    pub fn groupby_weekly(&self) -> Result<GroupedPriceRecord, Box<dyn std::error::Error>> {
        let bin_duration = chrono::Duration::weeks(1); // Default weekly duration

        // Modifying the start date to end of week
        let first_dt = *self
            .timestamp
            .first()
            .ok_or("Failed to get starting datetime.")?;
        let start_dt = chrono::DateTime::from_utc(
            chrono::NaiveDate::from_isoywd(first_dt.year(), first_dt.month(), chrono::Weekday::Mon)
                .and_hms(0, 0, 0),
            chrono::Utc,
        );
        let end_dt = *self
            .timestamp
            .last()
            .ok_or("Failed to get ending datetime.")?;
        let mut current_dt = start_dt;

        if bin_duration > end_dt - start_dt {
            return Err(
                "The bin duration should be greater than the intervals between the datetimes."
                    .into(),
            ); // From trait implemented from Box<dyn Error>
        }
        if bin_duration < self.timestamp[1] - self.timestamp[0] {
            return Err("Upsampling not supported at the moment.".into()); // From trait implemented from Box<dyn Error>
        }

        // Create the conditional for grouping
        let mut i: usize = 0;
        let mut group_indexes: Vec<std::ops::Range<usize>> = Vec::new();
        let mut current_group: Vec<usize> = Vec::with_capacity(5);
        let mut next_dt = current_dt + bin_duration;
        let mut binned_timestamps: Vec<chrono::DateTime<chrono::Utc>> = vec![start_dt];

        loop {
            current_dt = self.timestamp[i];
            if current_dt >= next_dt {
                group_indexes.push(std::ops::Range {
                    start: *current_group.first().unwrap(),
                    end: *current_group.last().unwrap() + 1,
                });
                current_group = Vec::with_capacity(5);
                binned_timestamps.push(next_dt);
                next_dt = next_dt + bin_duration;
            }
            current_group.push(i);
            i = i + 1;
            if i == self.timestamp.len() {
                if current_group.len() > 0 {
                    group_indexes.push(std::ops::Range {
                        start: *current_group.first().unwrap(),
                        end: *current_group.last().unwrap() + 1,
                    });
                }
                binned_timestamps.push(next_dt);
                break;
            }
        }

        let result = GroupedPriceRecord {
            ticker_symbol: self.ticker_symbol.clone(),
            groupby_duration: bin_duration,
            binned_timestamps: binned_timestamps,
            grouping_indexes: group_indexes.clone(),
            open_price: group_indexes
                .iter()
                .cloned()
                .map(|x| self.open_price.get(x).unwrap().to_vec())
                .collect::<Vec<Vec<f64>>>(),
            low_price: group_indexes
                .iter()
                .cloned()
                .map(|x| self.low_price.get(x).unwrap().to_vec())
                .collect::<Vec<Vec<f64>>>(),
            high_price: group_indexes
                .iter()
                .cloned()
                .map(|x| self.high_price.get(x).unwrap().to_vec())
                .collect::<Vec<Vec<f64>>>(),
            close_price: group_indexes
                .iter()
                .cloned()
                .map(|x| self.close_price.get(x).unwrap().to_vec())
                .collect::<Vec<Vec<f64>>>(),
            adj_close: group_indexes
                .iter()
                .cloned()
                .map(|x| self.adj_close.get(x).unwrap().to_vec())
                .collect::<Vec<Vec<f64>>>(),
            volume: group_indexes
                .iter()
                .cloned()
                .map(|x| self.volume.get(x).unwrap().to_vec())
                .collect::<Vec<Vec<i64>>>(),
            open_low_pricedelta: None,       // Created via builder patterns
            open_high_pricedelta: None,      // Created via builder patterns
            prevclose_low_pricedelta: None,  // Created via builder patterns
            prevclose_high_pricedelta: None, // Created via builder patterns
            currency: self.currency.clone(),
        };

        Ok(result)
    }
}

impl GroupedPriceRecord {
    pub fn with_prevclose_deltas(&mut self) -> &mut Self {
        match &self.prevclose_high_pricedelta {
            Some(i) => {}
            None => {
                let prevclose_vec = self.close_price.iter().flatten().collect::<Vec<&f64>>(); // Creating the Vec object that takes ownership of the data
                let prevclose_prices = prevclose_vec.split_last().unwrap().1; // creating a pointer to the slice object that references to the previously created vector
                let high_vec = self.high_price.iter().flatten().collect::<Vec<&f64>>(); // Creating the Vec object that takes ownership of the data
                let high_prices = high_vec.split_last().unwrap().1; // creating a pointer to the slice object that references to the previously created vector

                let mut prevclose_high_pricedelta = prevclose_prices
                    .iter()
                    .zip(high_prices.iter())
                    .map(|(&&x, &&y)| x - y)
                    .collect::<Vec<f64>>();
                prevclose_high_pricedelta.insert(0, f64::NAN); // first record will not have a previous closing price, and thus will not be computed.

                self.prevclose_high_pricedelta = Some(
                    self.grouping_indexes
                        .iter()
                        .cloned()
                        .map(|x| prevclose_high_pricedelta.get(x).unwrap().to_vec())
                        .collect::<Vec<Vec<f64>>>(),
                );
            }
        }

        match &self.prevclose_low_pricedelta {
            Some(i) => {}
            None => {
                let prevclose_vec = self.close_price.iter().flatten().collect::<Vec<&f64>>(); // Creating the Vec object that takes ownership of the data
                let prevclose_prices = prevclose_vec.split_last().unwrap().1; // creating a pointer to the slice object that references to the previously created vector
                let low_vec = self.low_price.iter().flatten().collect::<Vec<&f64>>(); // Creating the Vec object that takes ownership of the data
                let low_prices = low_vec.split_last().unwrap().1; // creating a pointer to the slice object that references to the previously created vector

                let mut prevclose_low_pricedelta = prevclose_prices
                    .iter()
                    .zip(low_prices.iter())
                    .map(|(&&x, &&y)| x - y)
                    .collect::<Vec<f64>>();
                prevclose_low_pricedelta.insert(0, f64::NAN); // first record will not have a previous closing price, and thus will not be computed.

                self.prevclose_low_pricedelta = Some(
                    self.grouping_indexes
                        .iter()
                        .cloned()
                        .map(|x| prevclose_low_pricedelta.get(x).unwrap().to_vec())
                        .collect::<Vec<Vec<f64>>>(),
                );
            }
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{num::ParseIntError, string::ParseError};

    #[test]
    fn test_get_prices() {
        let price_record = get_prices(
            "XLK",
            chrono::Utc.ymd(2022, 1, 1).and_hms(0, 0, 0),
            chrono::Utc.ymd(2022, 2, 1).and_hms(0, 0, 0),
        )
        .unwrap();
        println!("{:?}", [price_record]);
    }

    #[test]
    fn test_groupby_weekly() {
        let price_record = get_prices(
            "XLK",
            chrono::Utc.ymd(2022, 1, 5).and_hms(0, 0, 0),
            chrono::Utc.ymd(2022, 1, 28).and_hms(0, 0, 0),
        )
        .unwrap();
        let grouped_price_record = price_record.groupby_weekly().unwrap();
        println!("{:#?}", grouped_price_record);
    }
}
