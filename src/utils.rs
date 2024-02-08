use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Tz;

/// Get the next occurence of a date, that occurs after today (or today).
pub fn get_next_occurence(date: NaiveDate, timezone: Option<Tz>) -> DateTime<Local> {
    let today = Local::now();
    let current_year = today.year();

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
        if today.naive_local().date() <= birthday {
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
