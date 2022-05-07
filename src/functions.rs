//! Objective: Provide functionality for calculating useful metrics from the datasets.

use super::errors::AggregationError;

use itertools::Itertools;
use num_traits::{self, FromPrimitive, Num};

pub mod Grouping {
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
    use std::collections::HashMap;

    use super::*;

    // Defining custom types
    pub type AggregationResult<T, U> = std::collections::HashMap<T, U>;

    pub fn max<'a, T, U>(
        groupby: Grouping::GroupedBy<'a, T, U>,
    ) -> Result<AggregationResult<chrono::DateTime<chrono::Utc>, U>, AggregationError>
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
                    None => generic_zero, // Returns the generic zero if an iterator is empty.
                };
                (k, max_value)
            })
            .collect();

        Ok(result)
    }

    /// Values array in the GroupBy object must be a two-tuple, containing the (open value, close value) for each corresponding timestamp.
    pub fn openclose_delta<'a, T, U>(
        groupby: Grouping::GroupedBy<'a, T, (U, U)>,
    ) -> Result<AggregationResult<chrono::DateTime<chrono::Utc>, U>, AggregationError>
    where
        T: Ord,
        U: num_traits::Num + Copy,
    {
        let generic_zero = match <U as Num>::from_str_radix("0", 10) {
            Ok(i) => i,
            Err(_e) => unreachable!(),
        }; // Returns an equivalent zero value for the generic type.

        // Aggregation function implementation
        let result = groupby
            .into_iter()
            .map(|(k, v)| {
                let min_max_result = v.minmax_by_key(|x| x.0);
                match min_max_result {
                    itertools::MinMaxResult::NoElements => (k, generic_zero),
                    itertools::MinMaxResult::OneElement(record) => (k, record.1 .1 - record.1 .0),
                    itertools::MinMaxResult::MinMax(starting_record, ending_record) => {
                        (k, ending_record.1 .1 - starting_record.1 .0)
                    }
                }
            })
            .collect();

        Ok(result)
    }
}

pub fn percentile_from_sorted_array<T>(
    percentile: usize,
    arr: &[T],
) -> Result<T, std::num::TryFromIntError>
where
    T: num_traits::Float + num_traits::FromPrimitive,
{
    // Handling arrays with insufficient lengths

    // Handling invalid percentiles
    if percentile == 0 {
        return Ok(*arr.first().unwrap()); // Unwrap is unreachable due to the prior array checks.
    }

    if percentile == 100 {
        return Ok(*arr.last().unwrap()); // Unwrap is unreachable due to the prior array checks.
    }

    if percentile > 100 {
        eprintln!(
            "WARNING! Percentile value passed was greater than 100. Returning the maximum value."
        );
        return Ok(*arr.last().unwrap()); // Unwrap is unreachable due to the prior array checks
    }

    // Getting the length of the array
    let arr_length = arr.len();
    let max_index = arr_length - 1;
    let positional_index_floor = (max_index * percentile).div_euclid(100);

    if positional_index_floor == max_index {
        return Ok(arr[positional_index_floor]);
    } // positional_index_floor will always be <= max_index
    let positional_index_ceil: usize = positional_index_floor + 1; // Only defined after the checks

    // Getting the associated values in their corresponding positional indexes
    let floor_val = *arr.get(positional_index_floor).unwrap(); // No error is expected due to the prior bounds checking
    let ceil_val = *arr.get(positional_index_ceil).unwrap(); // No error is expected due to the prior bounds checking

    let euclid_remainder = u8::try_from((max_index * percentile).rem_euclid(100)).unwrap(); // Getting the remainder of the euclidean division - Will be bounded to a value between 0 (unsigned) and 100 (divisor value), hence conversion should be infallible.
    let multiplier = f32::try_from(euclid_remainder).unwrap() / 100.0; // Casting from u8 to f32 should be infallible.
    let delta_val = (ceil_val - floor_val) * T::from_f32(multiplier).unwrap(); // Since T will either be f32 or f64, casting should never fail regardless.

    Ok(floor_val + delta_val)
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

    // #[test]
    // fn visualize_aggregationfunctions_max() {
    //     let foo = datasets::structs::TickerInfo::new(
    //         "EEM",
    //         "2021-03-29 00:00:00",
    //         "2022-04-29 00:00:00",
    //         enums::Currency::Usd,
    //     )
    //     .unwrap();
    //     let bar = datasets::source_yahoo_finance(&foo).unwrap();
    //     let quxx = bar.get_high_prevclose_pricedelta_percentage();
    //     let baz = Grouping::groupby_weekly(bar.get_timestamps(), &quxx).unwrap();
    //     let qux = AggregationFunctions::max(baz);
    //     dbg!(qux.unwrap().values());
    // }

    #[test]
    fn visualize_openclose_delta() {
        let foo = datasets::structs::TickerInfo::new(
            "EEM",
            "2022-03-29 00:00:00",
            "2022-04-29 00:00:00",
            enums::Currency::Usd,
        )
        .unwrap();
        let bar = datasets::source_yahoo_finance(&foo).unwrap();
        let baz = bar
            .get_open_prices()
            .into_iter()
            .zip(bar.get_close_prices().into_iter())
            .map(|(a, b)| (*a, *b))
            .collect::<Vec<_>>();
        let quz = Grouping::groupby_weekly(bar.get_timestamps(), &baz).unwrap();
        let qux = AggregationFunctions::openclose_delta(quz).unwrap();
        let mut quuz = qux.into_values().collect::<Vec<_>>();
        quuz.sort_by(|a, b| a.partial_cmp(b).unwrap());
        dbg!(quuz);
    }

    // #[test]
    // fn visualize_percentile_from_sorted_array() {
    //     let foo = datasets::structs::TickerInfo::new(
    //         "EEM",
    //         "2022-01-01 00:00:00",
    //         "2022-04-18 00:00:00",
    //         enums::Currency::Usd,
    //     )
    //     .unwrap();
    //     let bar = datasets::source_yahoo_finance(&foo).unwrap();
    //     let quxx = bar.get_high_prevclose_pricedelta_percentage();
    //     let baz = Grouping::groupby_weekly(bar.get_timestamps(), &quxx).unwrap();
    //     let qux = AggregationFunctions::max(baz);
    //     let mut quuz = qux.unwrap().into_values().collect::<Vec<_>>();
    //     quuz.sort_by(|a, b| a.partial_cmp(b).unwrap());
    //     let barz = percentile_from_sorted_array(55, &quuz);
    //     dbg!(&quuz);
    //     dbg!(barz);
    // }
}
