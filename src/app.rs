use std::path::PathBuf;

use clap::Parser;
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliApp {
    pub filename: Option<PathBuf>,
    #[arg(short, long)]
    pub accont_filter: Vec<u16>,
    #[arg(short, long)]
    pub logger: bool,
}
