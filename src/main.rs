use datasets::traits::{Prices, Timestamps};

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

    // Extracting the corresponding values queued for evaluation
    let value_type = match input_args.get_price_type() {
        inputs::enums::PriceType::Open => ValueTypes::SingleValues(dataset.get_open_prices()),
        inputs::enums::PriceType::Close => ValueTypes::SingleValues(dataset.get_close_prices()),
        inputs::enums::PriceType::High => ValueTypes::SingleValues(dataset.get_high_prices()),
        inputs::enums::PriceType::Low => ValueTypes::SingleValues(dataset.get_low_prices()),
        inputs::enums::PriceType::OpenClose => {
            ValueTypes::DualValues(dataset.get_open_prices(), dataset.get_close_prices())
        }
    };

    // Performing the dispatch of the corresponding aggregation functions - Based on the ValueType variant constructed
    let result = match value_type {
        ValueTypes::SingleValues(values) => {
            let grouped =
                functions::Grouping::groupby_weekly(dataset.get_timestamps(), &values).unwrap();
            let result = functions::AggregationFunctions::max(grouped);
            result
        }
        ValueTypes::DualValues(values_first, values_second) => {
            let values = values_first
                .into_iter()
                .zip(values_second)
                .map(|(&a, &b)| (a, b))
                .collect::<Vec<_>>();
            let grouped =
                functions::Grouping::groupby_weekly(dataset.get_timestamps(), &values).unwrap();
            let result = functions::AggregationFunctions::openclose_delta(grouped);
            result
        }
    };
}

// fn process()
