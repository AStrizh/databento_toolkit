use time::{Date, Month, Weekday};
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
fn calculate_cl_expiration(year: i32, delivery_month: Month) -> Date {
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

/// Generates contract periods for CL from a start year to an end year (inclusive).
/// Each entry is a tuple: (symbol, download_start_date, download_end_date)
/// Download window is from 40 days before expiration to 3 days after.
pub fn generate_cl_contract_periods(start_year: i32, end_year: i32) -> Vec<(String, Date, Date)> {
    let mut periods = Vec::new();

    for year in start_year..=end_year {
        for &month in ALL_MONTHS.iter() {
            let expiry = calculate_cl_expiration(year, month);
            let start_date = expiry - time::Duration::days(40);
            let end_date = expiry + time::Duration::days(3);
            let symbol = format_cl_symbol(year, month);

            periods.push((symbol, start_date, end_date));
        }
    }

    periods
}
//-----------------------------------------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn test_cln3_expiration_date() {
        let delivery_month = Month::July;
        let year = 2025;

        let expected_expiration = date!(2025 - 06 - 22);
        let actual_expiration = calculate_cl_expiration(year, delivery_month);

        assert_eq!(expected_expiration, actual_expiration);
    }

    #[test]
    fn test_cln3_contract_symbol() {
        let delivery_month = Month::July;
        let year = 2023;

        let symbol = format_cl_symbol(year, delivery_month);
        assert_eq!(symbol, "CLN3");
    }

    #[test]
    fn test_generate_range_has_correct_count() {
        let periods = generate_cl_contract_periods(2022, 2023);
        assert_eq!(periods.len(), 24); // 2 years × 12 months
    }
}
