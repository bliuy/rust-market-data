//! Defining custom errors for the Market_Analysis crate.



#[derive(Debug)]
pub enum SourceDataError {
    ConnectionError(String),
    ParseError(String),
}

impl std::error::Error for SourceDataError {}

impl std::fmt::Display for SourceDataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Performing pattern matching
        match self {
            SourceDataError::ConnectionError(msg) => {
                std::fmt::write(formatter, format_args!("Error raised during the retrieval of source data: {}", msg))
            }
            SourceDataError::ParseError(msg) => {
                std::fmt::write(formatter, format_args!("Error raised during the parsing of source data: {}", msg))
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
        match returned_error{
            SourceDataError::ConnectionError(i) => {assert!(i==*"Test Connection Error.")},
            _ => panic!("Assertion failed.")
        }

    }

}
