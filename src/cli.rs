use std::path::PathBuf;

use crate::config::BirthdayDate;
use chrono_tz::Tz;
use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// The birthday file to use
    #[arg(short, long)]
    pub file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Adds a new entry
    Add {
        /// The name associated with the entry
        #[arg(short, long)]
        name: String,

        /// The date associated with the entry
        #[arg(short, long)]
        date: BirthdayDate,

        /// Optional timezone for the entry
        #[clap(short, long)]
        #[clap(value_parser = Tz::from_str_insensitive)]
        timezone: Option<Tz>,
    },
    // TODO: Add option to show raw timezone instead of duration until the birthday
    /// Lists entries
    List {
        /// Display only the closest n entries
        #[arg(short, long)]
        limit: Option<usize>,
    },
}
