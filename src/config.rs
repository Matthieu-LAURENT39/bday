use crate::utils;
use chrono::{DateTime, Local, NaiveDate};
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
    pub timezone: Option<Tz>,
    /// The next occurence of the date from today.  
    /// This is the date at midnight in the timezone of the entry,
    /// and changed to the local timezone.  
    /// If no timezone is specified, the local timezone is used.
    pub next_occurence: DateTime<Local>,
}

pub enum EntryError {
    TimezoneParseError(ParseError),
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
        let next_occurence = utils::get_next_occurence(toml_entry.date, timezone);
        Ok(Self {
            name: toml_entry.name,
            date: toml_entry.date,
            timezone,
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
        BaseDirs::new().and_then(|p: BaseDirs| Some(p.config_dir().join(CONFIG_FILE_NAME))),
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
