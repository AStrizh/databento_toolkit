use time::{Date, Duration, Month, Weekday};
use std::convert::TryFrom;

/// Maps Month enum to Futures month code letter.
fn futures_month_code(month: Month) -> &'static str {
    match month {
        Month::January => "F", Month::February => "G", Month::March => "H", Month::April => "J",
        Month::May => "K", Month::June => "M", Month::July => "N", Month::August => "Q",
        Month::September => "U", Month::October => "V", Month::November => "X", Month::December => "Z",
    }
}

const ALL_MONTHS: [Month; 12] = [
    Month::January, Month::February, Month::March, Month::April, Month::May, Month::June,
    Month::July, Month::August, Month::September, Month::October, Month::November, Month::December,
];

fn previous_month(month: Month, year: i32) -> (i32, Month) {
    use Month::*;
    match month {
        January => (year - 1, December),
        _ => {
            let prev_month_u8 = (month as u8) - 1;
            let prev_month = Month::try_from(prev_month_u8).expect("Invalid previous month");
            (year, prev_month)
        }
    }
}

fn energy_expiry(year: i32, delivery_month: Month) -> Date {
    let (exp_year, exp_month) = previous_month(delivery_month, year);
    let mut date = Date::from_calendar_date(exp_year, exp_month, 25).unwrap();
    let mut business_days = 0;

    while business_days < 3 {
        date = date.previous_day().unwrap();
        if !matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday) {
            business_days += 1;
        }
    }

    date
}

fn indices_expiry(year: i32, month: Month) -> Date {
    let mut count = 0;
    let mut date = Date::from_calendar_date(year, month, 1).unwrap();

    while count < 3 {
        if date.weekday() == Weekday::Friday {
            count += 1;
        }
        if count < 3 {
            date = date.next_day().unwrap();
        }
    }

    date
}

fn calculate_expiration_date(symbol: &str, year: i32, month: Month) -> Date {
    match symbol {
        "CL" | "NG" | "RB" | "HO" => energy_expiry(year, month),
        "ES" | "NQ" | "RTY" | "YM" => indices_expiry(year, month),
        _ => panic!("Unsupported symbol: {symbol}"),
    }
}

fn generate_energy_contracts(symbol: &str, start_date: Date, end_date: Date) -> Vec<(String, Date, Date)> {
    let mut periods = Vec::new();
    let mut current = start_date;

    while current <= end_date {
        for &month in ALL_MONTHS.iter() {
            let year = current.year();
            let expiry = calculate_expiration_date(symbol, year, month);
            if expiry >= start_date && expiry <= end_date {
                let code = futures_month_code(month);
                let symbol_code = format!("{}{}{}", symbol, code, year % 10);
                let start = expiry - Duration::days(40);
                let end = expiry + Duration::days(3);
                periods.push((symbol_code, start, end));
            }
        }
        current = Date::from_calendar_date(current.year() + 1, Month::January, 1).unwrap();
    }

    periods
}

fn generate_index_contracts(symbol: &str, start_date: Date, end_date: Date) -> Vec<(String, Date, Date)> {
    let mut periods: Vec<(String, Date, Date)> = Vec::new();

    // Start from the first contract expiry on or after start_date
    let mut current_year = start_date.year();
    let mut month_iter = [Month::March, Month::June, Month::September, Month::December].into_iter();

    // Initialize the first contract
    let mut previous_expiry = None;

    while current_year <= end_date.year() {
        for &month in &month_iter.clone().collect::<Vec<_>>() {
            let expiry = calculate_expiration_date(symbol, current_year, month);
            if expiry < start_date {
                continue;
            }
            if expiry > end_date {
                break;
            }

            let start = if let Some(prev_expiry) = previous_expiry {
                prev_expiry - Duration::days(10)
            } else {
                // First contract: assume 3-month default span
                expiry - Duration::days(90)
            };

            let code = futures_month_code(month);
            let symbol_code = format!("{}{}{}", symbol, code, current_year % 10);
            periods.push((symbol_code, start, expiry));

            previous_expiry = Some(expiry);
        }

        current_year += 1;
    }

    periods
}




pub fn generate_contract_periods(
    symbol: &str,
    start_date: Date,
    end_date: Date,
) -> Vec<(String, Date, Date)> {
    match symbol {
        "CL" | "NG" | "RB" | "HO" => generate_energy_contracts(symbol, start_date, end_date),
        "ES" | "NQ" | "RTY" | "YM" => generate_index_contracts(symbol, start_date, end_date),
        _ => panic!("Unsupported symbol: {symbol}"),
    }
}

//-----------------------------------------------------------------------------------------------------------------//
#[cfg(test)]
mod tests {
    use super::*;
    use time::{macros::date, Month};

    #[test]
    fn test_futures_month_code_mapping() {
        assert_eq!(futures_month_code(Month::February), "G");
        assert_eq!(futures_month_code(Month::December), "Z");
    }

    #[test]
    fn test_previous_month_regular_case() {
        let (year, month) = previous_month(Month::October, 2025);
        assert_eq!((year, month), (2025, Month::September));
    }

    #[test]
    fn test_previous_month_wraps_year() {
        let (year, month) = previous_month(Month::January, 2025);
        assert_eq!((year, month), (2024, Month::December));
    }

    #[test]
    fn test_energy_expiry_skips_weekends() {
        let expiry = energy_expiry(2023, Month::July);
        assert_eq!(expiry, date!(2023 - 06 - 21));
    }

    #[test]
    fn test_indices_expiry_third_friday() {
        let expiry = indices_expiry(2024, Month::June);
        assert_eq!(expiry, date!(2024 - 06 - 21));
    }

    #[test]
    fn test_generate_contract_periods_energy() {
        let periods = generate_contract_periods("NG", date!(2023 - 01 - 01), date!(2023 - 12 - 31));
        assert!(periods.len() >= 11);
        assert!(periods.iter().all(|(s, _, _)| s.starts_with("NG")));
    }

    #[test]
    fn test_generate_contract_periods_index() {
        let periods = generate_contract_periods("ES", date!(2023 - 01 - 01), date!(2023 - 12 - 31));
        assert_eq!(periods.len(), 4);
        assert!(periods.iter().all(|(s, _, _)| s.starts_with("ES")));
    }

    #[test]
    #[should_panic(expected = "Unsupported symbol")]
    fn test_generate_contract_periods_unsupported_symbol() {
        generate_contract_periods("XYZ", date!(2023 - 01 - 01), date!(2023 - 12 - 31));
    }

    #[test]
    fn test_es_contract_debug() {
        let periods = generate_contract_periods("ES", date!(2023 - 01 - 01), date!(2023 - 12 - 31));
        for (sym, start, end) in periods {
            println!("{}: {} to {}", sym, start, end);
        }
    }
}
