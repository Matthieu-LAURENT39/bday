use crate::utils;
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{ParseError, Tz};
use clap::error::Result;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{self, Path};

const CONFIG_FILE_NAME: &str = "rust-birthday.toml";

#[derive(Deserialize, Debug, Serialize)]
pub struct TomlEntry {
    pub name: String,
    pub date: NaiveDate,
    pub timezone: Option<String>,
}

pub struct Entry {
    pub name: String,
    pub date: NaiveDate,
    /// Considered as the local timezone if None
    pub timezone: Option<Tz>,
    /// The previous occurence of the date from today.
    /// If the date is today, this will be None.
    /// The time correspond be 23h59 in the requested timezone (aka the end of the date).
    pub prev_occurence: Option<DateTime<Local>>,
    /// The next occurence of the date from today.
    /// If the date is today, this will be None.
    /// The time correspond to midnight in the requested timezone (aka the begining of the date).
    pub next_occurence: Option<DateTime<Local>>,
}

pub enum EntryError {
    TimezoneParseError(ParseError),
}

/// Convert a naive DateTime (that is in the specified timezone) to the local timezone.
/// If no timezone is provided, the timezone used is the local timezone.
fn localize_naive_datetime(dt: NaiveDateTime, timezone: Option<Tz>) -> DateTime<Local> {
    match timezone {
        Some(tz) => tz.from_local_datetime(&dt).unwrap().with_timezone(&Local),
        None => Local.from_local_datetime(&dt).unwrap(),
    }
}

impl TryFrom<TomlEntry> for Entry {
    type Error = EntryError;

    fn try_from(toml_entry: TomlEntry) -> Result<Self, EntryError> {
        let timezone: Option<Tz> = match toml_entry.timezone {
            Some(tz) => match Tz::from_str_insensitive(&tz) {
                Ok(parsed_tz) => Some(parsed_tz),
                Err(e) => Err(EntryError::TimezoneParseError(e))?,
            },
            None => None,
        };

        // // The current time in the timezone of the entry, localised to UTC
        // let dt: DateTime<Utc> = match timezone {
        //     // Get current time in the timezone of the entry, then convert to UTC
        //     Some(tz) => tz
        //         .from_utc_datetime(&Utc::now().naive_utc())
        //         .with_timezone(&Utc),
        //     // Get current time in local timezone, then convert to UTC
        //     None => Local::now().with_timezone(&Utc),
        // };

        // The current date in the timezone of the entry
        let date_tz: NaiveDate = match timezone {
            Some(tz) => tz.from_utc_datetime(&Utc::now().naive_utc()).date_naive(),
            None => Local::now().naive_local().date(),
        };

        // We call it with the current time it is in the timezone of the entry
        let (prev_occurence, next_occurence) = match utils::find_prev_next_occurences(
            toml_entry.date.day(),
            toml_entry.date.month(),
            date_tz,
        ) {
            Some((prev, next)) => (
                Some(localize_naive_datetime(
                    prev.and_hms_opt(23, 59, 59).unwrap(),
                    timezone,
                )),
                Some(localize_naive_datetime(
                    next.and_hms_opt(0, 0, 0).unwrap(),
                    timezone,
                )),
            ),
            None => (None, None),
        };

        Ok(Self {
            name: toml_entry.name,
            date: toml_entry.date,
            timezone,
            prev_occurence,
            next_occurence,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub birthdays: Vec<TomlEntry>,
}

impl Default for Config {
    fn default() -> Self {
        Self { birthdays: vec![] }
    }
}

pub struct ConfigFile {
    pub path: path::PathBuf,
    pub config: Config,
}

impl Default for ConfigFile {
    fn default() -> Self {
        let path = BaseDirs::new()
            .map(|dirs| dirs.config_dir().join(CONFIG_FILE_NAME))
            // Fallback to a hardcoded path if BaseDirs::new() returns None
            .unwrap_or_else(|| path::PathBuf::from("~/.config/").join(CONFIG_FILE_NAME));

        Self {
            // Default to $XDG_CONFIG_HOME/birthdays.toml
            path,
            config: Config::default(),
        }
    }
}

pub enum LoadConfigError {
    /// A config file was found, but there was an error reading it
    IoError(std::io::Error),
    /// A config file was found, but there was an error parsing it
    TomlError(toml::de::Error),
    /// No valid config file found
    ConfigNotFound,
}

/// Load the config file  
/// The priority is in that order:
/// - ./birthdays.toml
/// - $XDG_CONFIG_HOME/birthdays.toml
/// - $HOME/.config/birthdays.toml
/// - $HOME/.birthdays.toml
pub fn load_config() -> Result<ConfigFile, LoadConfigError> {
    // Try various paths to find the config file
    for path in [
        //? ./birthdays.toml
        Some(Path::new(".").join(CONFIG_FILE_NAME)),
        //? $XDG_CONFIG_HOME/birthdays.toml.
        BaseDirs::new().map(|p: BaseDirs| p.config_dir().join(CONFIG_FILE_NAME)),
        //? $HOME/.config/birthdays.toml
        Some(Path::new("~/.config/").join(CONFIG_FILE_NAME)),
        //? $HOME/.birthdays.toml
        Some(Path::new("~/").join(".".to_owned() + CONFIG_FILE_NAME)),
    ]
    .iter()
    // Remove the None values
    .flatten()
    {
        if path.is_file() {
            let toml_str = fs::read_to_string(path).map_err(LoadConfigError::IoError)?;
            return toml::from_str(&toml_str)
                .map_err(LoadConfigError::TomlError)
                .map(|config| ConfigFile {
                    path: path.to_path_buf(),
                    config,
                });
        }
    }
    Err(LoadConfigError::ConfigNotFound)
}
