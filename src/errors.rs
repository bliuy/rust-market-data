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
}

impl std::error::Error for AggregationError {}

impl std::fmt::Display for AggregationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AggregationError::InconsistentLengthError(err) => {
                std::fmt::write(formatter, format_args!("Error occured in line {} due to inconsistencies observed between array lengths. See the error raised: {}", line!(), err))
            }
        }
    }
}

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
