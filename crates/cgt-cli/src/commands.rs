use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum Commands {
    /// Parse a transaction file and output JSON
    Parse {
        /// Input file path
        #[arg(required_unless_present = "schema")]
        file: Option<PathBuf>,

        /// Output JSON schema
        #[arg(long)]
        schema: bool,
    },
    /// Generate tax report
    Report {
        /// Input file path
        file: PathBuf,
    },
}
