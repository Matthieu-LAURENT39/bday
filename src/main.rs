use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone};
use chrono_humanize::HumanTime;
use chrono_tz::Tz;
use clap::{error::ErrorKind, Command, CommandFactory, Parser, Subcommand};
use core::time;
use prettytable::{format, row, Table};
use serde::{Deserialize, Serialize};
use std::{fs, process::exit};
use toml::value::Date;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author = "Matthieu LAURENT", version = "0.1", about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Adds a new entry
    Add {
        /// The name associated with the entry
        #[arg(short, long)]
        name: String,

        /// The date associated with the entry
        #[arg(short, long)]
        date: NaiveDate,

        /// Optional timezone for the entry
        #[clap(short, long)]
        timezone: Option<Tz>,
    },
    // TODO: Add "index" option to show indexes
    // TODO: Add option to show raw timezone instead of duration until the birthday
    List,
}

#[derive(Deserialize, Debug, Serialize)]
struct Entry {
    name: String,
    date: NaiveDate,
    timezone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    birthdays: Vec<Entry>,
}

/// Get the next occurence of a date
fn get_next_occurence(date: NaiveDate, timezone: Option<Tz>) -> DateTime<Local> {
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

fn main() {
    let cli = Cli::parse();

    let mut toml_entries: Config;
    if let Ok(toml_str) = fs::read_to_string("./test.toml") {
        match toml::from_str(&toml_str) {
            Ok(parsed) => toml_entries = parsed,
            Err(e) => Cli::command()
                .error(
                    ErrorKind::Io,
                    format!("Error parsing toml file:\n{}\nYou can delete the file, it will be recreated the next time you add a new birthday.", e),
                )
                // TODO: change the error code to 3
                // TODO: remove the "usage: " section that gets displayed
                .exit(),
        }
    } else {
        toml_entries = Config { birthdays: vec![] };
    }

    match &cli.command {
        Commands::Add {
            name,
            date,
            timezone,
        } => {
            // Add the entry to the config file
            println!(
                "Adding entry: {}, Date: {}{}",
                name,
                date,
                match timezone {
                    Some(tz) => format!(", Timezone: {}", tz.name()),
                    None => "".to_string(),
                }
            );
            let new_entry = Entry {
                name: name.clone(),
                date: *date,
                timezone: match timezone {
                    Some(tz) => Some(tz.name().to_string()),
                    None => None,
                },
            };
            toml_entries.birthdays.push(new_entry);
            let toml_str = toml::to_string(&toml_entries).expect("Error serializing toml");
            fs::write("./test.toml", toml_str).expect("Error writing toml file");
        }
        Commands::List => {
            if toml_entries.birthdays.is_empty() {
                eprintln!("No entries found, add some with the 'add' command.");
                exit(1);
            }
            // List all entries
            // for entry in toml_entries.birthdays.iter() {
            //     println!(
            //         "Name: {}, Date: {}, Timezone: {}",
            //         entry.name,
            //         entry.date,
            //         match &entry.timezone {
            //             Some(tz) => tz,
            //             None => "None",
            //         }
            //     );
            // }
            let mut table = Table::new();
            // TODO: Add option to use format::consts::FORMAT_CLEAN, for easy parsing
            table.set_format(*format::consts::FORMAT_BOX_CHARS);

            table.add_row(row!["Name", "Date", "Age", "In"]);
            for entry in toml_entries.birthdays.iter() {
                let timezone: Option<Tz>;
                if let Some(tz) = &entry.timezone {
                    match Tz::from_str_insensitive(&tz) {
                        Ok(parsed_tz) => timezone = Some(parsed_tz),
                        Err(e) => Cli::command()
                            .error(ErrorKind::Io, format!("Error parsing timezone: {}.", e))
                            // TODO: change the error code to 3
                            // TODO: remove the "usage: " section that gets displayed
                            .exit(),
                    }
                } else {
                    timezone = None;
                }
                let next_occurence = get_next_occurence(entry.date, timezone);
                let new_age = next_occurence.year() - entry.date.year();
                // TODO: Sort the entries by date of next occurence
                table.add_row(row![
                    entry.name,
                    // Chrono doesn't support locales yet
                    // entry.date.format("%C").to_string(),
                    entry.date.format("%d %B"), // TODO: Add option/config to customize the date format
                    format!("{} 🡒 {}", new_age - 1, new_age),
                    HumanTime::from(next_occurence - Local::now())
                ]);
            }

            table.printstd();
        }
    }
}
