use chrono::{Datelike, NaiveDate};

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
/// If the date is the same as the "date" argument, the result is None.  
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

#[cfg(test)]
mod tests {
    use super::find_prev_next_occurences;
    use chrono::NaiveDate;
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

    #[test_case(6, 2,
                NaiveDate::from_ymd_opt(2024, 2, 6).unwrap(),  
                None ; 
                "Birthday is same day as date")]
    #[test_case(1, 1,
                NaiveDate::from_ymd_opt(2024, 2, 2).unwrap(),  
                Some((
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 
                    NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                )) ; 
                "Birthday already happened this year")]
    #[test_case(6, 6,
        NaiveDate::from_ymd_opt(2024, 5, 5).unwrap(),  
        Some((
            NaiveDate::from_ymd_opt(2023, 6, 6).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 6).unwrap(),
        )) ; 
        "Birthday hasn't happened yet this year")]
    #[test_case(29, 2,
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),  
        Some((
            NaiveDate::from_ymd_opt(2023, 2, 28).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
        )) ; 
        "Birthday is feb 29th, and date is a leap year")]
    #[test_case(29, 2,
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),  
        Some((
            NaiveDate::from_ymd_opt(2022, 2, 28).unwrap(),
            NaiveDate::from_ymd_opt(2023, 2, 28).unwrap(),
        )) ; 
        "Birthday is feb 29th, and date is not a leap year")]
    fn test_find_prev_next_occurences(
        birthday_day: u32,
        birthday_month: u32,
        date: NaiveDate,
        expected: Option<(NaiveDate, NaiveDate)>,
    ) {
        assert_eq!(
            find_prev_next_occurences(birthday_day, birthday_month, date),
            expected
        );
    }
}
