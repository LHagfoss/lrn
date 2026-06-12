pub mod app;
pub mod args;
pub mod database;
pub mod ui;

use app::App;
use args::Args;
use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut app = App::new(args.vault_path);
    app.run()?;

    Ok(())
}
