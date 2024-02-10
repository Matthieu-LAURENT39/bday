use crate::utils;
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{ParseError, Tz};
use clap::error::Result;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::path::{self, Path};
use std::str::FromStr;
use std::{fmt, fs};

const CONFIG_FILE_NAME: &str = "rust-birthday.toml";

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct BirthdayDate {
    pub day: u32,
    pub month: u32,
    pub year: Option<i32>,
}

impl BirthdayDate {
    /// Get the date as a NaiveDate, using 2000 as default year if the year is not provided.  
    /// This is useful if you need a NaiveDate but don't care about the year.
    pub fn naive_date_safe_year(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year.unwrap_or(2000), self.month, self.day).unwrap()
    }
}

impl From<NaiveDate> for BirthdayDate {
    fn from(date: NaiveDate) -> Self {
        Self {
            day: date.day(),
            month: date.month(),
            year: Some(date.year()),
        }
    }
}

impl FromStr for BirthdayDate {
    type Err = &'static str;

    /// Parse a BirthdayDate from a string, in the format YYYY-MM-DD or MM-DD
    fn from_str(date: &str) -> Result<Self, Self::Err> {
        let separator = if date.contains('-') { '-' } else { '/' };
        let date_parts: Vec<&str> = date.split(separator).collect();

        // Determine positions of day, month, and year based on the format
        let (day, month, year) = match date_parts.len() {
            2 => {
                // DD/MM format
                if separator == '/' {
                    let day = date_parts[0].parse().map_err(|_| "Invalid day")?;
                    let month = date_parts[1].parse().map_err(|_| "Invalid month")?;
                    let year = None;
                    (day, month, year)
                } else {
                    return Err("Invalid date format, use DD/MM, DD/MM/YYYY, or YYYY-MM-DD");
                }
            }
            3 => {
                // YYYY-MM-DD format
                if separator == '-' {
                    let year = date_parts[0].parse().map_err(|_| "Invalid year")?;
                    let month = date_parts[1].parse().map_err(|_| "Invalid month")?;
                    let day = date_parts[2].parse().map_err(|_| "Invalid day")?;
                    (day, month, Some(year))
                }
                // DD/MM/YYYY format
                else {
                    let day = date_parts[0].parse().map_err(|_| "Invalid day")?;
                    let month = date_parts[1].parse().map_err(|_| "Invalid month")?;
                    let year = date_parts[2].parse().map_err(|_| "Invalid year")?;
                    (day, month, Some(year))
                }
            }
            _ => return Err("Invalid date format, use DD/MM, DD/MM/YYYY, or YYYY-MM-DD"),
        };

        // Check if the date is valid
        // We use 2020 as default as it is a leap year, so it can handle february 29th
        if NaiveDate::from_ymd_opt(year.unwrap_or(2000), month, day).is_none() {
            return Err("Invalid date");
        }

        Ok(Self { day, month, year })
    }
}

impl fmt::Display for BirthdayDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.year {
            Some(year) => write!(f, "{:02}/{:02}/{}", self.day, self.month, year),
            None => write!(f, "{:02}/{:02}", self.day, self.month),
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ConfigEntry {
    pub name: String,
    pub date: BirthdayDate,
    pub timezone: Option<String>,
}

pub struct Entry {
    pub name: String,
    pub date: BirthdayDate,
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

impl TryFrom<ConfigEntry> for Entry {
    type Error = EntryError;

    fn try_from(config_entry: ConfigEntry) -> Result<Self, EntryError> {
        let timezone: Option<Tz> = match config_entry.timezone {
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
            config_entry.date.day,
            config_entry.date.month,
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
            name: config_entry.name,
            date: config_entry.date,
            timezone,
            prev_occurence,
            next_occurence,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub birthdays: Vec<ConfigEntry>,
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
        //? $XDG_CONFIG_HOME/birthdays.toml
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
