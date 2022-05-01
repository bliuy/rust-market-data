//! Main purpose of this module is to provide input functionality
use std::io::Write;

use crate::errors::{self, InputError};

use self::structs::InputArgs;

pub mod enums {

    #[derive(Clone, Copy)]
    pub enum AggregationType {
        Max,
    }

    #[derive(Clone, Copy)]
    pub enum PriceType {
        Open,
        Close,
        High,
        Low,
        OpenClose,
    }

    #[derive(Clone, Copy)]
    pub enum AggregationPeriod {
        Weekly,
    }
}

pub mod structs {
    use super::*;

    pub struct InputArgs {
        pub(super) ticker_symbol: String,
        pub(super) price_type: enums::PriceType,
        pub(super) aggregation_period: enums::AggregationPeriod,
        pub(super) start_date: String,
        pub(super) end_date: String,
    }

    impl InputArgs {
        pub fn get_ticker_symbol(&self) -> &str {
            self.ticker_symbol.as_str()
        }

        pub fn get_price_type(&self) -> enums::PriceType {
            self.price_type // Copyable
        }

        pub fn get_aggregation_period(&self) -> enums::AggregationPeriod {
            self.aggregation_period // Copyable
        }

        pub fn get_start_date(&self) -> &str {
            self.start_date.as_str()
        }

        pub fn get_end_date(&self) -> &str {
            self.end_date.as_str()
        }
    }
}

/// Expected input format: [ticker symbol] ---[aggregation type]--- [price type] [aggregation period] [start date] [end date]
/// E.g. AAPL high_price weekly 2022-01-01 2022-02-01 --> Performs a weekly aggregation of the high prices in the period between 2022-01-01 and 2022-02-01.
pub fn get_input() -> Result<InputArgs, errors::InputError> {
    // Getting the user input
    let mut input_args = std::env::args().collect::<Vec<String>>();

    // Ensuring that the input string is of the correct length
    if input_args.len() > 5 {
        return Err(InputError::ExcessiveArgsError(format!(
            "Expected 5 arguments, got {}.",
            input_args.len()
        )));
    } else if input_args.len() < 5 {
        return Err(InputError::InsufficientArgsError(format!(
            "Expected 5 arguments, got {}.",
            input_args.len()
        )));
    }

    // Matching on the PriceType
    let price_type = match input_args.get(1).unwrap() as &str {
        "open_price" => enums::PriceType::Open,
        "close_price" => enums::PriceType::Close,
        "high_price" => enums::PriceType::High,
        "low_price" => enums::PriceType::Low,
        "open_close_price" => enums::PriceType::OpenClose,
        _ => {
            return Err(errors::InputError::InvalidPriceType(
                "Value passed for the price type is invalid.".to_string(),
            ))
        }
    };

    // Matching on the PriceType
    let aggregation_period = match input_args.get(2).unwrap() as &str {
        "weekly" => enums::AggregationPeriod::Weekly,
        _ => {
            return Err(errors::InputError::InvalidAggregationPeriod(format!(
                "Value passed for the aggregation period is invalid."
            )))
        }
    };

    // Construction of the remaining variables - Moving the Strings out of the vec and replacing with empty Strings
    let ticker_symbol = std::mem::take(input_args.get_mut(0).unwrap());
    let start_date = std::mem::take(input_args.get_mut(3).unwrap());
    let end_date = std::mem::take(input_args.get_mut(4).unwrap());

    Ok(structs::InputArgs {
        ticker_symbol,
        price_type,
        aggregation_period,
        start_date,
        end_date,
    })
}

pub fn stdin() -> std::io::Result<String> {
    let mut s = String::new();
    let _ = std::io::stdout().flush(); // Performing flushing of the output buffer
    std::io::stdin().read_line(&mut s)?; // Reading a single line of input
                                         // For newline cases whereby the end of the string is appended as "\r\n" (Windows) or "\n" (Unix), remove these characters
    if let Some('\n') = s.chars().next_back() {
        s.pop(); // Removing '\n' newline character if present
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop(); // Removing '\r' carriage return character if present
    }
    Ok(s)
}
