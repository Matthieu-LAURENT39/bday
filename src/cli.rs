use chrono::NaiveDate;
use chrono_tz::Tz;
use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
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
        date: NaiveDate,

        /// Optional timezone for the entry
        #[clap(short, long)]
        timezone: Option<Tz>,
    },
    // TODO: Add "index" option to show indexes
    // TODO: Add option to show raw timezone instead of duration until the birthday
    /// Lists entries
    List {
        /// Display only the closest n entries
        #[arg(short, long)]
        limit: Option<usize>,
    },
}
