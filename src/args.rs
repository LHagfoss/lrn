use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "A lightning-fast TUI Markdown personal wiki")]
pub struct Args {
    #[arg(short, long, default_value = ".")]
    pub vault_path: PathBuf,
}
