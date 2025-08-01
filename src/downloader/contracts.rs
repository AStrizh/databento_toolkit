use time::{Date, Duration, Month, Weekday};
use std::convert::TryFrom;

/// Maps Month enum to Futures month code letter.
/// E.g. January → "F", February → "G", ..., December → "Z"
fn futures_month_code(month: Month) -> &'static str {
    match month {
        Month::January => "F", Month::February => "G", Month::March => "H", Month::April => "J",
        Month::May => "K", Month::June => "M", Month::July => "N", Month::August => "Q",
        Month::September => "U", Month::October => "V", Month::November => "X", Month::December => "Z",
    }
}

/// Constant array of all months used to iterate over full year cycles
const ALL_MONTHS: [Month; 12] = [
    Month::January, Month::February, Month::March, Month::April, Month::May, Month::June,
    Month::July, Month::August, Month::September, Month::October, Month::November, Month::December,
];

/// Converts a Month to the previous calendar month (accounting for year rollover)
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

/// Returns a crude oil (CL) contract symbol like "CLN3" given year and month
fn format_cl_symbol(year: i32, delivery_month: Month) -> String {
    let code = futures_month_code(delivery_month);
    let short_year = year % 10; // Databento wants 1-digit year format
    format!("CL{}{}", code, short_year)
}

/// Calculates the expiration date for CL contracts based on CME rules:
/// "Trading terminates 3 business days before the 25th of the month prior to delivery."
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
    // Start at 1st of the month and find the 3rd Friday
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


/// Generates contract periods for CL from a start year to an end year (inclusive).
/// Each entry is a tuple: (symbol, download_start_date, download_end_date)
/// Download window is from 40 days before expiration to 3 days after.
// pub fn generate_cl_contract_periods(start_year: i32, end_year: i32) -> Vec<(String, Date, Date)> {
//     let mut periods = Vec::new();
//
//     for year in start_year..=end_year {
//         for &month in ALL_MONTHS.iter() {
//             let expiry = calculate_cl_expiration(year, month);
//             let start_date = expiry - time::Duration::days(40);
//             let end_date = expiry + time::Duration::days(3);
//             let symbol = format_cl_symbol(year, month);
//
//             periods.push((symbol, start_date, end_date));
//         }
//     }
//
//     periods
// }

fn calculate_expiration_date(symbol: &str, year: i32, month: Month) -> Date {
    match symbol {
        // Energy commodities:
        "CL" | "NG" | "RB" | "HO" => energy_expiry(year, month),

        // Equity indices:
        "ES" | "NQ" | "RTY" | "YM" => indices_expiry(year, month),

        // Add more cases as needed...
        _ => panic!("Unsupported symbol: {symbol}"),
    }
}



pub fn generate_contract_periods(
    start_date: Date,
    end_date: Date,
    symbols: &[&str],
) -> Vec<(String, Date, Date)> {
    let mut all_periods = vec![];

    for &symbol in symbols {
        match symbol {
            "CL" => {
                let mut current = start_date;
                while current <= end_date {
                    for month in ALL_MONTHS.iter() {
                        let year = current.year();
                        let expiry = calculate_expiration_date(symbol, year, *month);
                        if expiry >= start_date && expiry <= end_date {
                            let code = futures_month_code(*month);
                            let symbol_code = format!("{symbol}{}{}", code, year % 10);
                            let start = expiry - Duration::days(40);
                            let end = expiry + Duration::days(3);
                            all_periods.push((symbol_code, start, end));
                        }
                    }
                    current = Date::from_calendar_date(current.year() + 1, Month::January, 1).unwrap();
                }
            }
            _ => panic!("Unsupported symbol: {symbol}"),
        }
    }

    all_periods
}




//-----------------------------------------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn test_cl_expiration_date() {
        let delivery_month = Month::October;
        let year = 2025;

        let expected_expiration = date!(2025 - 09 - 22);
        let actual_expiration = energy_expiry(year, delivery_month);

        assert_eq!(expected_expiration, actual_expiration);
    }

    #[test]
    fn test_cl_contract_symbol() {
        let delivery_month = Month::September;
        let year = 2026;

        let symbol = format_cl_symbol(year, delivery_month);
        assert_eq!(symbol, "CLU6");
    }

    #[test]
    fn test_generate_range_has_correct_count() {
        let periods = generate_contract_periods(date!(2022 - 01 - 01), date!(2023 - 01 - 01), &["CL"]);
        assert_eq!(periods.len(), 12);
        assert_eq!(periods.len(), 24); // 2 years × 12 months
    }

    #[test]
    fn test_previous_month_handles_year_boundary() {
        let (year, month) = previous_month(Month::January, 2024);
        assert_eq!((year, month), (2023, Month::December));
    }

    #[test]
    fn test_futures_month_code_mapping() {
        assert_eq!(futures_month_code(Month::February), "G");
        assert_eq!(futures_month_code(Month::December), "Z");
    }

    #[test]
    fn test_generate_cl_contract_periods_first_last_symbol() {
        let periods = generate_contract_periods(date!(2022 - 01 - 01), date!(2023 - 01 - 01),&["CL"]);
        let first_symbol = &periods.first().unwrap().0;
        let last_symbol = &periods.last().unwrap().0;
        assert_eq!(first_symbol, "CLF2");
        assert_eq!(last_symbol, "CLZ3");
    }


    #[test]
    fn test_previous_month_wraps_year() {
        let (y, m) = previous_month(Month::January, 2024);
        assert_eq!(y, 2023);
        assert_eq!(m, Month::December);
    }

    #[test]
    fn test_calculate_cl_expiration_weekend() {
        // June 25th, 2023 is a Sunday so expiration should fall on the prior
        // Wednesday (21st).
        let expiry = energy_expiry(2023, Month::July);
        assert_eq!(expiry, date!(2023 - 06 - 21));
    }

    #[test]
    fn test_first_period_contents() {
        let periods =  generate_contract_periods(date!(2022 - 01 - 01), date!(2023 - 12 - 31),&["CL"]);
        let (symbol, start, end) = &periods[0];
        assert_eq!(symbol, "CLF2");
        assert_eq!(*start, date!(2021 - 11 - 12));
        assert_eq!(*end, date!(2021 - 12 - 25));
    }
}
