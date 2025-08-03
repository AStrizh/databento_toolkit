// This module is adapted from egui_extras::DatePickerButton
// Copyright the egui authors: https://github.com/emilk/egui
// 
// This local patch addresses a UX issue where the date picker popup 
// would close immediately after selecting a year, month, or day from a dropdown,
// without waiting for explicit confirmation.
// 
// Changes made here ensure that the popup only closes when the user
// clicks "Save", presses Escape, or clicks outside the calendar.



mod button;
mod popup;

pub use button::CustomDatePickerButton;
use chrono::{Datelike as _, Duration, NaiveDate, Weekday};

#[derive(Debug)]
struct Week {
    number: u8,
    days: Vec<NaiveDate>,
}

fn month_data(year: i32, month: u32) -> Vec<Week> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("Could not create NaiveDate");
    let mut start = first;
    while start.weekday() != Weekday::Mon {
        start = start.checked_sub_signed(Duration::days(1)).unwrap();
    }
    let mut weeks = vec![];
    let mut week = vec![];
    while start < first || start.month() == first.month() || start.weekday() != Weekday::Mon {
        week.push(start);

        if start.weekday() == Weekday::Sun {
            weeks.push(Week {
                number: start.iso_week().week() as u8,
                days: std::mem::take(&mut week),
            });
        }
        start = start.checked_add_signed(Duration::days(1)).unwrap();
    }

    weeks
}
