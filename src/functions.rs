//! Objective: Provide functionality for calculating useful metrics from the datasets.

use super::datasets;
use super::errors::AggregationError;
use chrono;
use itertools::Itertools;
use num_traits::{self, Num};

mod grouping {
    use super::*;
    use chrono::{DateTime, Datelike};
    use std::{iter::Zip, slice::Iter};

    /// Function will aggregate the dataset on a weekly basis.
    pub fn groupby_weekly<'a, T, U>(
        timestamps: &'a [T],
        values: &'a [U],
    ) -> Result<
        itertools::GroupBy<
            chrono::DateTime<chrono::Utc>,
            Zip<Iter<'a, T>, Iter<'a, U>>,
            fn(&(&T, &U)) -> chrono::DateTime<chrono::Utc>,
        >,
        AggregationError,
    >
    where
        T: chrono::Datelike,
        U: Ord + Copy + num_traits::Num,
    {
        // Validating that the lengths of the arrays are equal.
        if timestamps.len() != values.len() {
            return Err(AggregationError::InconsistentLengthError(format!(
                "Length of the timestamps array: {} \n Length of the values array: {}",
                timestamps.len(),
                values.len()
            )));
        }

        // Defining the grouping function
        fn _grouping_function<X, Y>(x: &(&X, &Y)) -> chrono::DateTime<chrono::Utc>
        where
            X: chrono::Datelike,
            Y: PartialOrd,
        {
            let (timestamp, value) = x;
            let current_week = timestamp.iso_week().week0();
            let week: chrono::DateTime<chrono::Utc> = chrono::Utc::today()
                .and_hms(0, 0, 0)
                .with_ordinal0(current_week * 7)
                .unwrap();
            week
        } // NOTE: A closure (anonymous type) will not work as it cannot be defined within the , since the GroupBy struct field will require an actual specific type, while impl Traits are only viable for function signatures.
          // Following error will be returned if closure is used:
          // mismatched types
          // expected struct `itertools::GroupBy<_, std::iter::Zip<std::slice::Iter<'a, _>, std::slice::Iter<'a, _>>, for<'r, 's, 't0> fn(&'r (&'s T, &'t0 U)) -> u32>`
          //    found struct `itertools::GroupBy<_, std::iter::Zip<std::slice::Iter<'_, _>, std::slice::Iter<'_, _>>, [closure@src/functions.rs:49:61: 50:42]>`

        // Performing casting of the function - Coercing from Function Item (idenfication via distinct type) to Function Pointer (identification via stored address)
        let grouping_function =
            _grouping_function as fn(&(&T, &U)) -> chrono::DateTime<chrono::Utc>;

        // The above step helps to avoid the following error:
        // expected struct `itertools::GroupBy<_, std::iter::Zip<std::slice::Iter<'a, _>, std::slice::Iter<'a, _>>, for<'r, 's, 't0> fn(&'r (&'s DateTime<Utc>, &'t0 f32)) -> _>`
        // found struct `itertools::GroupBy<_, std::iter::Zip<std::slice::Iter<'_, _>, std::slice::Iter<'_, _>>, for<'r, 's, 't0> fn(&'r (&'s DateTime<Utc>, &'t0 f32)) -> _ {grouping_function}>`

        // Performing the aggregation on a weekly basis
        let result = timestamps.iter().zip(values).group_by(grouping_function);

        Ok(result)
    }
}

mod AggregationFunctions {

    use super::*;

    // Defining custom types
    type AggregationResult<T: chrono::Datelike, U: Ord> = std::collections::HashMap<T, U>;

    fn max<'a, T, U>(
        groupby: itertools::GroupBy<
            chrono::DateTime<chrono::Utc>,
            std::iter::Zip<std::slice::Iter<'a, T>, std::slice::Iter<'a, U>>,
            fn(&(&T, &U)) -> chrono::DateTime<chrono::Utc>,
        >,
    ) -> AggregationResult<chrono::DateTime<chrono::Utc>, U>
    where
        T: chrono::Datelike,
        U: Ord + Copy + num_traits::Num,
    {
        // Creating new AggregationResult object
        let mut result: AggregationResult<chrono::DateTime<chrono::Utc>, U> =
            AggregationResult::new();

        // Processing of the individual groups
        let result: AggregationResult<chrono::DateTime<chrono::Utc>, U> = groupby
            .into_iter()
            .map(|(k, v)| {
                let max_value = match v.map(|(x, &y)| y).max() {
                    Some(i) => i,
                    None => {
                        let zero = match <U as Num>::from_str_radix("0", 10) {
                            Ok(j) => j,
                            Err(e) => unreachable!(),
                        };
                        zero
                    }
                };
                (k, max_value)
            })
            .collect();

        result
    }
}
