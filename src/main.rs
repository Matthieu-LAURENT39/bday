use chrono::{DateTime, Datelike, Local};
use chrono_humanize::HumanTime;
use clap::error::Result;
use clap::{error::Error, error::ErrorKind, Command, CommandFactory, Parser, Subcommand};
use prettytable::{format, row, Table};
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

    let mut conf_file: config::ConfigFile = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => match e {
            // Use a default config if no config file is found
            config::LoadConfigError::ConfigNotFound => config::ConfigFile::default(),
            // TODO: Use clap to display the error message
            config::LoadConfigError::IoError(e) => {
                eprintln!("Error reading config file: {}", e);
                exit(3);
            }
            config::LoadConfigError::TomlError(e) => {
                eprintln!("Error parsing toml file:\n{}\nYou can delete the file, it will be recreated the next time you add a new birthday.", e);
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
            println!(
                "Adding entry: {}, Date: {}{}",
                name,
                date,
                match timezone {
                    Some(tz) => format!(", Timezone: {}", tz.name()),
                    None => "".to_string(),
                }
            );
            let new_entry = config::TomlEntry {
                name: name.clone(),
                date: *date,
                timezone: match timezone {
                    Some(tz) => Some(tz.name().to_string()),
                    None => None,
                },
            };
            conf_file.config.birthdays.push(new_entry);
            let toml_str = toml::to_string(&conf_file.config).expect("Error serializing toml");
            fs::write(conf_file.path, toml_str).expect("Error writing toml file");
        }
        cli::Commands::List { limit } => {
            if conf_file.config.birthdays.is_empty() {
                eprintln!("No entries found, add some with the 'add' command.");
                exit(0);
            }

            let now: DateTime<Local> = Local::now();

            // Parse the TomlEntry to Entry
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
                        cli::Cli::command()
                            .error(ErrorKind::Io, format!("Error parsing timezone: {}.", e))
                            // TODO: change the error code to 3
                            // TODO: remove the "usage: " section that gets displayed
                            .exit()
                    }
                },
            };

            // Sort the entries by date of next occurence
            // TODO: Maybe move this earlier to we don't have to use mut on entries
            entries.sort_by(|a, b| a.next_occurence.cmp(&b.next_occurence));

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
            table.set_titles(row![b => "Name", "Date", "Age", "In"]);
            let iter: Box<dyn Iterator<Item = &config::Entry>> = match limit {
                Some(limit) => Box::new(entries.iter().take(*limit)),
                None => Box::new(entries.iter()),
            };
            for entry in iter {
                let new_age = entry.next_occurence.year() - entry.date.year();
                table.add_row(row![
                    entry.name,
                    // Chrono doesn't support locales yet
                    // entry.date.format("%C").to_string(),
                    entry.date.format("%d %B"), // TODO: Add option/config to customize the date format
                    format!("{} 🡒 {}", new_age - 1, new_age),
                    HumanTime::from(entry.next_occurence - now)
                ]);
            }

            table.printstd();
        }
    }
}
