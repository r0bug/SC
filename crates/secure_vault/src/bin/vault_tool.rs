use clap::{Parser, Subcommand};
use secure_vault::{decrypt_to_string, encrypt_env_file};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "SagensContact vault utility")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Encrypt a plaintext env file into a vault file
    Encrypt {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(short, long)]
        key: String,
    },
    /// Decrypt a vault file to stdout or file
    Decrypt {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        key: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Encrypt { input, output, key } => {
            encrypt_env_file(input, output, &key)?;
            println!("Vault written successfully");
        }
        Command::Decrypt { input, key, output } => {
            let plaintext = decrypt_to_string(input, &key)?;
            if let Some(path) = output {
                std::fs::write(path, plaintext)?;
            } else {
                println!("{}", plaintext);
            }
        }
    }

    Ok(())
}
