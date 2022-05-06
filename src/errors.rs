//! Defining custom errors for the Market_Analysis crate.

#[derive(Debug)]
pub enum SourceDataError {
    ConnectionError(String),
    ParseError(String),
    MissingDataError(String),
}

impl std::error::Error for SourceDataError {}

impl std::fmt::Display for SourceDataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Performing pattern matching
        match self {
            SourceDataError::ConnectionError(msg) => std::fmt::write(
                formatter,
                format_args!("Error raised during the retrieval of source data: {}", msg),
            ),
            SourceDataError::ParseError(msg) => std::fmt::write(
                formatter,
                format_args!("Error raised during the parsing of source data: {}", msg),
            ),
            SourceDataError::MissingDataError(msg) => std::fmt::write(
                formatter,
                format_args!("Missing data found! See the error raised: {}", msg),
            ),
        }
    }
}

#[derive(Debug)]
pub enum InitializationError {
    TickerInfoInitializationError(String),
}

impl std::error::Error for InitializationError {}

impl std::fmt::Display for InitializationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Performing pattern matching
        match self {
            InitializationError::TickerInfoInitializationError(err) => {
                std::fmt::write(formatter, format_args!("Error occured during the initialization of the TickerInfo struct object. See the error raised: {}", err))
            }
        }
    }
}

#[derive(Debug)]
pub enum AggregationError {
    InconsistentLengthError(String),
    ComparisonError(String),
}

impl std::error::Error for AggregationError {}

impl std::fmt::Display for AggregationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AggregationError::InconsistentLengthError(err) => {
                std::fmt::write(formatter, format_args!("Error occured in line {} due to inconsistencies observed between array lengths. See the error raised: {}", line!(), err))
            }
            AggregationError::ComparisonError(err) => {
                std::fmt::write(formatter, format_args!("Error occured in line {} due to failure in comparison of the values. See the error raised: {}", line!(), err))
            }
        }
    }
}

#[derive(Debug)]
pub enum InputError {
    ExcessiveArgsError(String),
    InsufficientArgsError(String),
    InvalidAggregationType(String),
    InvalidPriceType(String),
    InvalidAggregationPeriod(String),
    IOError(String),
}

impl std::error::Error for InputError {}

impl std::fmt::Display for InputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InputError::ExcessiveArgsError(err) => {
                std::fmt::write(formatter, format_args!("Error occured in line {} due to too many arguments being present. See the error raised: {}", line!(), err))
            },
            InputError::InsufficientArgsError(err) => {
                std::fmt::write(formatter, format_args!("Error occured in line {} due to to insufficient args passed. See the error raised: {}", line!(), err))
            },
            InputError::InvalidAggregationType(err) => {
                std::fmt::write(formatter, format_args!("Error occured in line {} due to to an invalid option passed as the aggregation type parameter. See the error raised: {}", line!(), err))
            },
            InputError::InvalidPriceType(err) => {
                std::fmt::write(formatter, format_args!("Error occured in line {} due to to an invalid option passed as the price type parameter. See the error raised: {}", line!(), err))
            },
            InputError::InvalidAggregationPeriod(err) => {
                std::fmt::write(formatter, format_args!("Error occured in line {} due to to an invalid option passed as the aggregation period parameter. See the error raised: {}", line!(), err))
            },
            InputError::IOError(err) => {
                std::fmt::write(formatter, format_args!("Error occured in line {} due to to an I/O error. See the error raised: {}", line!(), err))
            },
        }
    }
}

impl From<std::io::Error> for InputError {
    fn from(e: std::io::Error) -> Self {
        let error_msg = format!("{}", e);
        InputError::IOError(error_msg)
    }
} // Handling the conversion from the io library error into the custom error defined in this crate.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn SourceDataError() {
        let test_func = || SourceDataError::ConnectionError("Test Connection Error.".to_string());
        let returned_error = test_func();
        match returned_error {
            SourceDataError::ConnectionError(i) => {
                assert!(i == *"Test Connection Error.")
            }
            _ => panic!("Assertion failed."),
        }
    }
}
