use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod http_client_generator;
mod ts_generator;

#[derive(Parser)]
#[command(name = "uniffi-bindgen")]
#[command(about = "Binding generator for Aurelia")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(name = "ts")]
    Ts {
        #[arg(long)]
        out_dir: PathBuf,
    },
    #[command(name = "http-client")]
    HttpClient {
        #[arg(long)]
        out_dir: PathBuf,
    },
    #[command(name = "all")]
    All {
        #[arg(long)]
        out_dir: PathBuf,
    },
    #[command(name = "generate")]
    Generate {
        #[arg(long)]
        library: PathBuf,
        #[arg(long)]
        language: String,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        no_format: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ts { out_dir } => {
            ts_generator::generate_typescript_bindings(&out_dir)
                .expect("Failed to generate TypeScript bindings");
        }
        Commands::HttpClient { out_dir } => {
            http_client_generator::generate_http_client(&out_dir)
                .expect("Failed to generate HTTP client");
        }
        Commands::All { out_dir } => {
            ts_generator::generate_typescript_bindings(&out_dir)
                .expect("Failed to generate TypeScript bindings");
            http_client_generator::generate_http_client(&out_dir)
                .expect("Failed to generate HTTP client");
        }
        Commands::Generate {
            library: _,
            language: _,
            config: _,
            out_dir: _,
            no_format: _,
        } => {
            uniffi::uniffi_bindgen_main();
        }
    }
}
