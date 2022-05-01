//! This module contains all the functions required to perform a HTTP request
use super::errors;

pub fn blocking_reqwest(url: &str) -> Result<reqwest::blocking::Response, errors::SourceDataError> {
    let _response = reqwest::blocking::get(url);
    let response = match _response {
        Ok(resp) => Ok(resp),
        Err(_err) => Err(errors::SourceDataError::ConnectionError(format!(
            "Error when attempting to connect to the following url: \n {}",
            url
        ))),
    };

    response
}
