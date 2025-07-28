use time::{Date, Month};

/// Returns the Databento symbol (e.g., CLU25) active on a given date.
pub fn front_month_symbol_on(date: Date) -> String {
    // Implement logic per symbol type
    // Crude Oil (CL) expires ~20th, so front-month rolls ~6 trading days earlier.
    let (year, month) = (date.year(), date.month());

    // Estimate the expiration around 20th
    let expiration_day = 20;
    let rollover_lead_days = 6;

    let expiration_date = Date::from_calendar_date(year, month, expiration_day).unwrap();
    let rollover_date = expiration_date - time::Duration::days(rollover_lead_days);

    let target_month = if date < rollover_date {
        month
    } else {
        month.next()
    };

    let code = futures_month_code(target_month);
    let yy = if target_month == Month::January { year + 1 } else { year };
    let short_year = (yy % 100) as u8;

    format!("CL{}{}", code, short_year)
}

fn futures_month_code(month: Month) -> &'static str {
    match month {
        Month::January => "F", Month::February => "G", Month::March => "H", Month::April => "J",
        Month::May => "K", Month::June => "M", Month::July => "N", Month::August => "Q",
        Month::September => "U", Month::October => "V", Month::November => "X", Month::December => "Z",
    }
}

pub fn generate_contract_periods(start: Date, end: Date) -> Vec<(String, Date, Date)> {
    let mut current = start;
    let mut last_symbol = String::new();
    let mut periods = vec![];

    while current <= end {
        let symbol = front_month_symbol_on(current);
        if symbol != last_symbol {
            // Define padded range
            let range_start = current - time::Duration::days(10);
            let range_end = current + time::Duration::days(30); // generous end
            periods.push((symbol.clone(), range_start, range_end));
            last_symbol = symbol;
        }
        current += time::Duration::days(1);
    }
    periods
}