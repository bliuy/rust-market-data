//! Objective: Provide functionality for calculating useful metrics from the datasets.

use super::datasets;
use super::errors::AggregationError;
use chrono;
use itertools::Itertools;

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
            u32,
            Zip<Iter<'a, T>, Iter<'a, U>>,
            fn(&(&T, &U)) -> u32,
        >,
        AggregationError,
    > where T: chrono::Datelike, U: PartialOrd {
        // Validating that the lengths of the arrays are equal.
        if timestamps.len() != values.len() {
            return Err(AggregationError::InconsistentLengthError(format!(
                "Length of the timestamps array: {} \n Length of the values array: {}",
                timestamps.len(),
                values.len()
            )));
        }

        // Defining the grouping function
        fn _grouping_function<X,Y>(x: &(&X, &Y)) -> u32 where X: chrono::Datelike, Y: PartialOrd {
            let (timestamp, value) = x;
            timestamp.iso_week().week0()
        } // NOTE: A closure will not work, since the GroupBy struct field will require an actual specific type, while impl Traits are only viable for function signatures.

        // Performing casting of the function - Coercing from Function Item (idenfication via distinct type) to Function Pointer (identification via stored address)
        let grouping_function =
            _grouping_function as fn(&(&T, &U)) -> u32;

        // The above step helps to avoid the following error:
        // expected struct `itertools::GroupBy<_, std::iter::Zip<std::slice::Iter<'a, _>, std::slice::Iter<'a, _>>, for<'r, 's, 't0> fn(&'r (&'s DateTime<Utc>, &'t0 f32)) -> _>`
        // found struct `itertools::GroupBy<_, std::iter::Zip<std::slice::Iter<'_, _>, std::slice::Iter<'_, _>>, for<'r, 's, 't0> fn(&'r (&'s DateTime<Utc>, &'t0 f32)) -> _ {grouping_function}>`

        // Performing the aggregation on a weekly basis
        let result = timestamps.iter().zip(values).group_by(grouping_function);

        Ok(result)
    }
}
