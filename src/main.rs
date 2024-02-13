use chrono::{DateTime, Datelike, Local};
use chrono_humanize::HumanTime;
// use clap::error::Result;
// use clap::{error::Error, error::ErrorKind, Command, CommandFactory, Parser, Subcommand};
use clap::{error::ErrorKind, CommandFactory, Parser};
use directories::BaseDirs;
use prettytable::{format, row, Table};
use std::path::PathBuf;
use std::{fs, process::exit};

mod cli;
mod config;
mod utils;

/// Exit codes:  
/// 0: Success. Note that this is still returned if no entries are found, but
///    the program will print an error message to stderr in that case, leaving stdout empty.  
/// 2: Invalid command, or other clap parsing error  
/// 3: Error reading or parsing the config file  
fn main() {
    let cli = cli::Cli::parse();

    //? Defaults to $XDG_CONFIG_HOME/bday.toml
    let conf_path: PathBuf = cli.file.unwrap_or_else(|| {
        BaseDirs::new()
            .map(|p| p.config_dir().join("bday.toml"))
            .expect("Error getting the default birthday file path.\nYou can always use a custom birthday file with the --file option.")
    });

    let mut conf_file: config::ConfigFile = match config::load_config(&conf_path) {
        Ok(cfg) => cfg,
        Err(e) => match e {
            // Use a default config if no config file is found
            config::LoadConfigError::ConfigNotFound => config::ConfigFile {
                path: conf_path,
                config: config::Config::default(),
            },
            config::LoadConfigError::IoError(e) => {
                let _ = cli::Cli::command()
                    .error(ErrorKind::Io, format!("Error reading config file: {}", e))
                    // TODO: remove the "usage: " section that gets displayed
                    .print();
                exit(3);
            }
            config::LoadConfigError::TomlError(e) => {
                let _ = cli::Cli::command()
                    .error(ErrorKind::Io, format!("Error parsing the birthday file:\n{}\nYou can delete the file, it will be recreated the next time you add a new birthday.", e))
                    // TODO: remove the "usage: " section that gets displayed
                    .print();
                exit(3);
            }
        },
    };

    match &cli.command {
        cli::Commands::Add {
            name,
            date,
            timezone,
        } => {
            // Add the entry to the config file
            let new_entry = config::ConfigEntry {
                name: name.clone(),
                date: *date,
                timezone: timezone.as_ref().map(|tz| tz.name().to_string()),
            };
            conf_file.config.birthdays.push(new_entry);
            let toml_str =
                toml::to_string(&conf_file.config).expect("Error serializing birthday file");
            fs::write(conf_file.path, toml_str).expect("Error writing birthday file");
            println!(
                "Added entry for {}, born: {}{}",
                name,
                date,
                match timezone {
                    Some(tz) => format!(" (Timezone: {})", tz.name()),
                    None => "".to_string(),
                }
            );
        }
        cli::Commands::List { limit } => {
            if conf_file.config.birthdays.is_empty() {
                eprintln!("No entries found, add some with the 'add' command.");
                exit(0);
            }

            let now: DateTime<Local> = Local::now();

            // Parse the ConfigEntry to Entry
            let mut entries: Vec<config::Entry> = match conf_file
                .config
                .birthdays
                .into_iter()
                .map(config::Entry::try_from)
                .collect()
            {
                Ok(entries) => entries,
                Err(e) => match e {
                    config::EntryError::TimezoneParseError(e) => {
                        let _ = cli::Cli::command()
                            .error(ErrorKind::Io, format!("Error parsing timezone: {}.", e))
                            // TODO: remove the "usage: " section that gets displayed
                            .print();
                        exit(3);
                    }
                },
            };

            // Sort the entries by date of next occurence
            // TODO: Maybe move this earlier to we don't have to use mut on entries
            entries.sort_by(|a, b| b.next_occurence.cmp(&a.next_occurence));

            let mut table = Table::new();
            // table.set_format(*format::consts::FORMAT_BOX_CHARS);
            table.set_format(
                format::FormatBuilder::new()
                    .column_separator('│')
                    .borders('│')
                    .separators(
                        &[format::LinePosition::Top],
                        format::LineSeparator::new('─', '┬', '╭', '╮'),
                    )
                    .separators(
                        &[format::LinePosition::Intern],
                        format::LineSeparator::new('─', '┼', '├', '┤'),
                    )
                    .separators(
                        &[format::LinePosition::Bottom],
                        format::LineSeparator::new('─', '┴', '╰', '╯'),
                    )
                    .padding(1, 1)
                    .build(),
            );

            // Makes the header bold
            table.set_titles(row![b => "#", "Name", "Date", "Age", "In"]);
            let iter: Box<dyn Iterator<Item = &config::Entry>> = match limit {
                Some(limit) => Box::new(entries.iter().take(*limit)),
                None => Box::new(entries.iter()),
            };
            for (index, entry) in iter.enumerate() {
                let new_age: Option<i32> = entry
                    .date
                    .year
                    .map(|y| entry.next_occurence.unwrap_or(Local::now()).year() - y);

                table.add_row(row![
                    index + 1,
                    entry.name,
                    // Chrono doesn't support locales yet
                    // entry.date.format("%C").to_string(),
                    entry.date.naive_date_safe_year().format("%d %B"),
                    match new_age {
                        Some(age) => format!("{} 🡒 {}", age - 1, age),
                        None => "?".to_string(),
                    },
                    match entry.next_occurence {
                        Some(dt) => HumanTime::from(dt - now).to_string(),
                        None => "Today!".to_string(),
                    }
                ]);
            }

            table.printstd();
        }
    }
}
