use crate::commands::list_keyboards;
use clap::{Parser, Subcommand};

mod commands;

// Control LEDS via HID
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // List all connected Logitech HID devices
    ListKeyboards,
    // place holder for more..
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::ListKeyboards => list_keyboards()?,
    }
    Ok(())
}
