//! Objective: Provide functionality for calculating useful metrics from the datasets.

use super::datasets;
use super::errors::AggregationError;
use chrono;
use itertools::Itertools;

mod grouping {
    use super::*;
    use chrono::{DateTime, Datelike};
    use std::{iter::Zip, slice::Iter};

    // fn func(x:i32) -> i32 {x+1}

    // Box<dyn FnMut(&(&chrono::DateTime<chrono::Utc>, &f32)) u32>

    // let func = |x: i32|->i32 {x+1};
    type grouped<'a> = itertools::GroupBy<
        u32,
        Zip<Iter<'a, chrono::DateTime<chrono::Utc>>, Iter<'a, f32>>,
        fn(&(&chrono::DateTime<chrono::Utc>, &f32)) -> u32,
    >;

    /// Function will aggregate the dataset on a weekly basis.
    pub fn groupby_weekly<'a, T, U>(
        timestamps: &'a [chrono::DateTime<chrono::Utc>],
        values: &'a [f32],
    ) -> Result<grouped<'a>, AggregationError> {
        // Validating that the lengths of the arrays are equal.
        if timestamps.len() != values.len() {
            return Err(AggregationError::InconsistentLengthError(format!(
                "Length of the timestamps array: {} \n Length of the values array: {}",
                timestamps.len(),
                values.len()
            )));
        }

        // Defining the grouping function
        fn _grouping_function(x: &(&chrono::DateTime<chrono::Utc>, &f32)) -> u32 {
            let (timestamp, value) = x;
            timestamp.iso_week().week0()
        }

        // Performing casting of the function - Coercing from Function Item (idenfication via distinct type) to Function Pointer (identification via stored address)
        let grouping_function =
            _grouping_function as fn(&(&chrono::DateTime<chrono::Utc>, &f32)) -> u32;

        // The above step helps to avoid the following error:
        // expected struct `itertools::GroupBy<_, std::iter::Zip<std::slice::Iter<'a, _>, std::slice::Iter<'a, _>>, for<'r, 's, 't0> fn(&'r (&'s DateTime<Utc>, &'t0 f32)) -> _>`
        // found struct `itertools::GroupBy<_, std::iter::Zip<std::slice::Iter<'_, _>, std::slice::Iter<'_, _>>, for<'r, 's, 't0> fn(&'r (&'s DateTime<Utc>, &'t0 f32)) -> _ {grouping_function}>`

        // let zipped_iterator = timestamps.into_iter().zip(values);

        // let result = zipped_iterator.group_by(|&(timestamp, value)| timestamp.iso_week().week0());

        // Performing the aggregation on a weekly basis
        let result = timestamps.iter().zip(values).group_by(grouping_function);

        Ok(result)
        // todo!()
    }
}
