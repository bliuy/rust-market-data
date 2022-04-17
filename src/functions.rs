//! Objective: Provide functionality for calculating useful metrics from the datasets.

use super::errors::AggregationError;

use itertools::Itertools;
use num_traits::{self, Num};

mod Grouping {
    use super::*;
    use chrono::Datelike;
    use std::{iter::Zip, slice::Iter};

    pub type GroupedBy<'a, T, U> = itertools::GroupBy<
        chrono::DateTime<chrono::Utc>,
        Zip<Iter<'a, T>, Iter<'a, U>>,
        fn(&(&T, &U)) -> chrono::DateTime<chrono::Utc>,
    >;

    /// Function will aggregate the dataset on a weekly basis.
    pub fn groupby_weekly<'a, T, U>(
        timestamps: &'a [T],
        values: &'a [U],
    ) -> Result<GroupedBy<'a, T, U>, AggregationError>
    where
        T: chrono::Datelike,
        U: PartialOrd + Copy + num_traits::Num,
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
            let (timestamp, _value) = x;
            let naive_week = chrono::NaiveDate::from_isoywd(
                timestamp.year(),
                timestamp.iso_week().week(),
                chrono::Weekday::Mon,
            );

            chrono::Date::from_utc(naive_week, chrono::Utc).and_hms(0, 0, 0)
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

pub mod AggregationFunctions {

    use std::cmp::Ordering;

    use super::*;

    // Defining custom types
    type AggregationResult<T, U> = std::collections::HashMap<T, U>;

    pub fn max<'a, T, U>(
        groupby: Grouping::GroupedBy<'a, T, U>,
    ) -> AggregationResult<chrono::DateTime<chrono::Utc>, U>
    where
        T: chrono::Datelike,
        U: PartialOrd + Copy + num_traits::Num,
    {
        // Creating new AggregationResult object
        let _result: AggregationResult<chrono::DateTime<chrono::Utc>, U> = AggregationResult::new();
        let generic_zero = match <U as Num>::from_str_radix("0", 10) {
            Ok(i) => i,
            Err(_e) => unreachable!(),
        }; // Returns an equivalent zero value for the generic type.

        // Processing of the individual groups
        let result: AggregationResult<chrono::DateTime<chrono::Utc>, U> = groupby
            .into_iter()
            .map(|(k, v)| {
                let max_value = match v.map(|(_x, &y)| y).max_by(|a, b| match a.partial_cmp(b) {
                    Some(i) => i,
                    None => {
                        if a.partial_cmp(&generic_zero).is_none() {
                            Ordering::Less // Returns "b" as "a" is the NaN value
                        } else {
                            Ordering::Greater // Returns "a" as "b" is the NaN value
                        }
                    }
                }) {
                    Some(i) => i,
                    None => generic_zero,
                };
                (k, max_value)
            })
            .collect();

        result
    }


}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::datasets;
    use crate::datasets::traits::*;
    use crate::enums;

    #[test]
    fn visualize_groupby_weekly() {
        let foo = datasets::structs::TickerInfo::new(
            "AAPL",
            "2022-01-01 00:00:00",
            "2022-04-01 00:00:00",
            enums::Currency::Usd,
        )
        .unwrap();
        let bar = datasets::source_yahoo_finance(&foo).unwrap();
        let baz = Grouping::groupby_weekly(bar.get_timestamps(), bar.get_high_prices()).unwrap();
        for qux in baz.into_iter() {
            dbg!(&qux.0);
            for quux in qux.1 {
                dbg!(quux);
            }
        }
    }

    #[test]
    fn visualize_aggregationfunctions_max() {
        let foo = datasets::structs::TickerInfo::new(
            "XLK",
            "2022-01-01 00:00:00",
            "2022-04-01 00:00:00",
            enums::Currency::Usd,
        )
        .unwrap();
        let bar = datasets::source_yahoo_finance(&foo).unwrap();
        let quxx = bar.get_high_prevclose_pricedelta_percentage();
        let baz = Grouping::groupby_weekly(bar.get_timestamps(), &quxx).unwrap();
        let qux = AggregationFunctions::max(baz);
        dbg!(qux);
    }
}
