use clap::{Parser, Subcommand};

mod commands;
mod keyboard;

use crate::commands::{list_keyboards, print_device};

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
    // Open a specific keyboard and print its info
    PrintDevice {
        // Serial number of the keyboard (if omitted, emits the first supported device)
        #[arg(long)]
        serial: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::ListKeyboards => list_keyboards()?,
        Commands::PrintDevice { serial } => print_device(serial)?,
    }
    Ok(())
}
