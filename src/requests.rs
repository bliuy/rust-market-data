//! This module contains all the functions required to perform a HTTP request
use super::errors;
use reqwest;


pub fn blocking_reqwest(url: &str) -> Result<reqwest::blocking::Response, errors::SourceDataError> {
    let _response = reqwest::blocking::get(url);
    let response = match _response {
        Ok(resp) => Ok(resp),
        Err(err) => Err(errors::SourceDataError::ConnectionError(format!(
            "Error when attempting to connect to the following url: \n {}",
            url
        ))),
    };

    response
}