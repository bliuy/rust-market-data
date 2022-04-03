//! Purpose of this module is to provide parsing capabilities

use super::errors;



pub fn parse_blocking_response_bytes(
    response: reqwest::blocking::Response,
) -> Result<Vec<u8>, errors::SourceDataError> {
    // Attempt to parse the input response object into bytes
    

    match response.bytes() {
        Ok(i) => Ok(i.into_iter().collect::<Vec<u8>>()),
        Err(_) => Err(errors::SourceDataError::ParseError(
            "Unable to parse the response object!".to_string(),
        )),
    }
}

pub mod serde_parsers {
    use chrono::TimeZone;


    /// Used for parsing standard datetimes into chrono::Datetime<chrono::Utc> output.
    /// Input datetime should be in the following format: %Y-%m-%d
    pub fn parsing_std_dates<'de, D>(
        deserializer: D,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

        struct DatetimeVisitor;

        impl<'de> serde::de::Visitor<'de> for DatetimeVisitor {
            type Value = chrono::DateTime<chrono::Utc>;

            fn expecting(&self, fmtter: &mut std::fmt::Formatter) -> std::fmt::Result {
                fmtter.write_str("Cannot parse the input data into a datetime type.")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                
                chrono::Utc
                    .datetime_from_str(value, DATETIME_FORMAT)
                    .map_err(|_x| {
                        serde::de::Error::custom(format!(
                            "Error while parsing the following datetime value: {}",
                            value
                        ))
                    })
            }
        }

        match deserializer.deserialize_str(DatetimeVisitor) {
            Ok(i) => Ok(Some(i)),
            Err(e) => {
                eprintln!("Error occured while parsing datetime: {}", e); // Print the error to stderr. 
                Ok(None) // Instead of bubbling up an error, return a None instead.
            }
        }
    }
}
