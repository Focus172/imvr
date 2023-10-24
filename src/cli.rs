use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(last = true)]
    pub files: Vec<PathBuf>,
}
