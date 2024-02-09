use core::time;

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;

// The current time in the timezone, localised to UTC
// If no timezone is provided, the timezone used is the local timezone
fn now_utc(timezone: Option<Tz>) -> DateTime<Utc> {
    match timezone {
        // Get current time in the timezone of the entry, then convert to UTC
        Some(tz) => tz
            .from_utc_datetime(&Utc::now().naive_utc())
            .with_timezone(&Utc),
        // Get current time in local timezone, then convert to UTC
        None => Local::now().with_timezone(&Utc),
    }
}

/// Add a number of years to a date.
/// Handles the february 29th case, by returning february 28th on non-leap years.
fn make_date_safe(year: i32, month: u32, day: u32) -> NaiveDate {
    // The match None branch is mainly to handle the february 29th case
    // I can't think of any other case where with_year would return None, so i'm not handling it
    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => date,
        // Try the previous day (so feb 29th becomes feb 28th)
        None => NaiveDate::from_ymd_opt(year, month, day - 1).unwrap(),
    }
}

/// Get the previous and next occurences of a date, relative to a given date.  
/// If the date is today, the result is None.  
pub fn find_prev_next_occurences(
    day: u32,
    month: u32,
    date: NaiveDate,
) -> Option<(NaiveDate, NaiveDate)> {
    let current_year = date.year();

    let curr_year_birthday = make_date_safe(current_year, month, day);

    // If the birthday is today, return None
    if date.day() == day && date.month() == month {
        return None;
    }

    // If the birthday already happened this year
    if curr_year_birthday < date {
        let next_year_birthday = make_date_safe(current_year + 1, month, day);
        Some((curr_year_birthday, next_year_birthday))
    // If the birthday hasn't happened yet this year
    } else {
        let prev_year_birthday = make_date_safe(current_year - 1, month, day);
        Some((prev_year_birthday, curr_year_birthday))
    }
}

/*
/// Get the next occurence of a date, that occurs after today (or today).
/// The time will be midnight in the requested timezone, or in the local timezone if none is provided.
pub fn get_next_occurence(date: NaiveDate, timezone: Option<Tz>) -> DateTime<Local> {
    // Now in the local timezone, converted to UTC
    let now: DateTime<Utc> = Local::now().into();
    let current_year = now.year();

    let mut offset = 0;
    loop {
        // The match None branch is mainly to handle the february 29th case
        // I can't think of any other case where with_year would return None, so i'm not handling it
        let birthday = match date.with_year(current_year + offset) {
            Some(date) => date,
            // Try the previous day (so feb 29th becomes feb 28th)
            None => (date - Duration::days(1))
                .with_year(current_year + offset)
                .unwrap(),
        };

        // If the birthday is today or in the future, return it
        if now <= birthday {
            // Find the time for midnight in the timezone of the entry
            return match timezone {
                Some(tz) => {
                    tz.from_local_datetime(&birthday.and_time(NaiveTime::MIN))
                        // Documentation is very unclear as to what can cause it to return Err
                        .unwrap()
                        .with_timezone(&Local)
                }
                None => Local
                    .from_local_datetime(&birthday.and_time(NaiveTime::MIN))
                    .unwrap(),
            };
        }

        offset += 1;
    }
}
*/

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, Local, NaiveDate};
    use chrono_tz::TZ_VARIANTS;
    use test_case::test_case;

    #[test]
    fn test_make_date_safe() {
        // Test a leap year
        assert_eq!(
            super::make_date_safe(2020, 2, 29),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        // Test a non-leap year
        assert_eq!(
            super::make_date_safe(2021, 2, 29),
            NaiveDate::from_ymd_opt(2021, 2, 28).unwrap()
        );
    }

    // #[test]
    // fn test_get_next_occurence_coherent() {
    //     // Both are inclusive
    //     let start_date = NaiveDate::from_ymd_opt(1700, 1, 1).unwrap();
    //     let end_date = Local::now().date_naive();

    //     // Try every date between start_date and end_date, with every possible timezone.
    //     // I assume that birthdays can't be in the future.
    //     // This might take a while.
    //     for tz in TZ_VARIANTS.iter() {
    //         let mut current_date = start_date;
    //         while current_date <= end_date {
    //             let next_occurence = super::get_next_occurence(current_date, Some(*tz));

    //             // Add one day
    //             current_date += chrono::Duration::days(1);
    //         }
    //     }
    // }
}
