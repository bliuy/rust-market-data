use datasets::traits::{Prices, Timestamps};
use functions::AggregationFunctions;

mod datasets;
mod enums;
mod errors;
mod functions;
mod inputs;
mod parsers;
mod requests;

enum ValueTypes<'a, T> {
    SingleValues(&'a [T]),
    DualValues(&'a [T], &'a [T]),
}

fn main() {
    // Getting the input from the user
    let mut input_arg_result = inputs::get_input();
    while let Err(e) = input_arg_result {
        println!("{}", e);
        input_arg_result = inputs::get_input();
    } // Requests the user to perform continuous inputs until no errors are raised.

    // Unwrapping the inputs into an InputArgs object
    let input_args: inputs::structs::InputArgs = match input_arg_result {
        Ok(i) => i,
        Err(e) => unreachable!(),
    };

    // Creating the TickerInfo object
    let ticker_info = match datasets::structs::TickerInfo::new(
        input_args.get_ticker_symbol(),
        input_args.get_start_date(),
        input_args.get_end_date(),
        enums::Currency::Usd,
    ) {
        Ok(i) => i,
        Err(e) => {
            println!(
                "Following error raised when attempting to parse the provided information: {}",
                e
            );
            return (); // Termination of the main function
        }
    };

    // Performing dataset retrieval
    let dataset = match datasets::source_yahoo_finance(&ticker_info) {
        Ok(i) => i,
        Err(e) => {
            println!(
                "Following error raised when attempting to parse the provided information: {}",
                e
            );
            return (); // Termination of the main function
        }
    };

    // Dispatching based on the PriceType
    let _get_price_values = |x| match x {
        inputs::enums::BasePriceType::Open => dataset.get_open_prices(),
        inputs::enums::BasePriceType::Close => dataset.get_close_prices(),
        inputs::enums::BasePriceType::High => dataset.get_high_prices(),
        inputs::enums::BasePriceType::Low => dataset.get_low_prices(),
    };
    let _get_price_type = |x| match x {
        inputs::enums::BasePriceType::Open => "Open".to_string(),
        inputs::enums::BasePriceType::Close => "Close".to_string(),
        inputs::enums::BasePriceType::High => "High".to_string(),
        inputs::enums::BasePriceType::Low => "Low".to_string(),
    };

    let summarized_result = match input_args.get_price_type() {
        inputs::enums::PriceType::SinglePrice(price_type) => {
            let price_values = _get_price_values(price_type);
            let timestamps = dataset.get_timestamps();

            // Grouping into period groups
            let grouped = match input_args.get_aggregation_period() {
                inputs::enums::AggregationPeriod::Weekly => {
                    match functions::Grouping::groupby_weekly(timestamps, price_values) {
                        Ok(i) => i,
                        Err(e) => {
                            println!("Error encountered! See the following error raised: {}.", e);
                            return ();
                        }
                    }
                }
            };

            // Identifying the summary function
            let summary_func = loop {
                // Handling the input
                let input_string =
                    match inputs::stdin("Input the summarization function to be applied:") {
                        Ok(i) => i,
                        Err(e) => {
                            println!("Following error encountered: {}. \n Please try again!", e);
                            continue;
                        }
                    };
                // Dispatching the corresponding function
                match input_string.as_str() {
                    "max" => {
                        break functions::AggregationFunctions::max::<
                            chrono::DateTime<chrono::Utc>,
                            f32,
                        >
                    }
                    _ => {
                        println!(
                            "Unknown summary function provided. Please input a valid summary function!"
                        );
                        continue;
                    }
                };
            };

            summary_func(grouped)
        }
        inputs::enums::PriceType::DualPrice(price_type_first, price_type_second) => {
            let price_values_first = _get_price_values(price_type_first);
            let price_values_second = _get_price_values(price_type_second);
            let price_type_first_str = _get_price_type(price_type_first);
            let price_type_second_str = _get_price_type(price_type_second);

            // Determining the aggregation method
            let price_values = loop {
                // Handling the input
                let prompt_msg = format!("Enter the corresponding number of the aggregation function to be applied: \n
                1 - Price delta. This will be calculated by using the formula for each timestamp: ({} price - {} price).\n
                2 - Price delta percentage. This will be calculated by using the formula for each timestamp: (({} price - {} price)/{} price) * 100.\n
                3 - Trailing Price delta. This will be calculated by using the formula for each timestamp: ({} price - previous {} price).\n
                4 - Trailing Price delta percentage. This will be calculated by using the formula for each timestamp: (({} price - previous {} price)/ previous {} price) * 100.\n
                ", price_type_first_str
                , price_type_second_str
                , price_type_first_str
                , price_type_second_str
                , price_type_second_str
                , price_type_first_str
                , price_type_second_str
                , price_type_first_str
                , price_type_second_str
                , price_type_second_str);
                let input_string = match inputs::stdin(&prompt_msg) {
                    Ok(i) => i,
                    Err(e) => {
                        println!("Following error encountered: {}. \n Please try again!", e);
                        continue;
                    }
                };
                match input_string.as_str() {
                    "1" => {
                        let length = price_values_first.len();
                        let result = price_values_first
                            .into_iter()
                            .zip(price_values_second)
                            .map(|(first_val, second_val)| first_val - second_val)
                            .collect::<Vec<_>>();
                        break result;
                    }
                    "2" => {
                        let length = price_values_first.len();
                        let result = price_values_first
                            .into_iter()
                            .zip(price_values_second)
                            .map(|(first_val, second_val)| {
                                ((first_val - second_val) / (second_val)) * 100.0
                            })
                            .collect::<Vec<_>>();
                        break result;
                    }
                    "3" => {
                        let length = price_values_first.len();
                        let mut result = vec![f32::NAN];
                        price_values_first[1..]
                            .into_iter()
                            .zip(&price_values_second[..length - 1])
                            .for_each(|(first_val, second_val)| {
                                result.push((first_val - second_val))
                            });
                        break result;
                    }
                    "4" => {
                        let length = price_values_first.len();
                        let mut result = vec![f32::NAN];
                        price_values_first[1..]
                            .into_iter()
                            .zip(&price_values_second[..length - 1])
                            .for_each(|(first_val, second_val)| {
                                result.push(((first_val - second_val) / (second_val)) * 100.0)
                            });
                        break result;
                    }
                    _ => {
                        println!(
                            "Unknown summary function provided. Please input a valid summary function!"
                        );
                        continue;
                    }
                }
            };

            // Grouping into period groups
            let timestamps = dataset.get_timestamps();
            let grouped = match input_args.get_aggregation_period() {
                inputs::enums::AggregationPeriod::Weekly => {
                    match functions::Grouping::groupby_weekly(timestamps, &price_values) {
                        Ok(i) => i,
                        Err(e) => {
                            println!("Error encountered! See the following error raised: {}.", e);
                            return ();
                        }
                    }
                }
            };

            // Identifying the summary function
            let summary_func = loop {
                // Handling the input
                let input_string =
                    match inputs::stdin("Input the summarization function to be applied:") {
                        Ok(i) => i,
                        Err(e) => {
                            println!("Following error encountered: {}. \n Please try again!", e);
                            continue;
                        }
                    };
                // Dispatching the corresponding function
                match input_string.as_str() {
                    "max" => {
                        break functions::AggregationFunctions::max::<
                            chrono::DateTime<chrono::Utc>,
                            f32,
                        >
                    }
                    _ => {
                        println!(
                            "Unknown summary function provided. Please input a valid summary function!"
                        );
                        continue;
                    }
                };
            };

            summary_func(grouped)
        }
    };

    // Displaying the summarized results
    match summarized_result {
        Ok(result) => result.into_iter().for_each(|(datetime, value)| {
            println!("Datetime: {} ---> Aggregated value: {}", datetime, value)
        }),
        Err(e) => {
            println!("Error raised: {}", e);
            return ();
        }
    }

    // // Extracting the corresponding values queued for evaluation
    // let value_type = match input_args.get_price_type() {
    //     inputs::enums::PriceType::Open => ValueTypes::SingleValues(dataset.get_open_prices()),
    //     inputs::enums::PriceType::Close => ValueTypes::SingleValues(dataset.get_close_prices()),
    //     inputs::enums::PriceType::High => ValueTypes::SingleValues(dataset.get_high_prices()),
    //     inputs::enums::PriceType::Low => ValueTypes::SingleValues(dataset.get_low_prices()),
    //     inputs::enums::PriceType::OpenClose => {
    //         ValueTypes::DualValues(dataset.get_open_prices(), dataset.get_close_prices())
    //     }
    // };

    // // Performing the dispatch of the corresponding aggregation functions - Based on the ValueType variant constructed
    // let _aggregation_result = match value_type {
    //     ValueTypes::SingleValues(values) => {
    //         let grouped = match input_args.get_aggregation_period() {
    //             Weekly => functions::Grouping::groupby_weekly(dataset.get_timestamps(), &values),
    //         }
    //         .unwrap(); // unwrap as placeholder
    //         let result = functions::AggregationFunctions::max(grouped);
    //         result
    //     }
    //     ValueTypes::DualValues(values_first, values_second) => {
    //         let values = values_first
    //             .into_iter()
    //             .zip(values_second)
    //             .map(|(&a, &b)| (a, b))
    //             .collect::<Vec<_>>();
    //         let grouped =
    //             functions::Grouping::groupby_weekly(dataset.get_timestamps(), &values).unwrap();
    //         let result = functions::AggregationFunctions::openclose_delta(grouped);
    //         result
    //     }
    // };

    // let aggregation_result = match _aggregation_result {
    //     Ok(i) => i,
    //     Err(e) => {
    //         println!(
    //             "Following error raised during the aggregation process: {}",
    //             e
    //         );
    //         return ();
    //     }
    // };

    // // Displaying the aggregated values
    // aggregation_result
    //     .into_iter()
    //     .for_each(|(datetime, values)| {
    //         println!("Timestamp: {}, Aggregated Value: {}", datetime, values)
    //     });
}

// fn process()
