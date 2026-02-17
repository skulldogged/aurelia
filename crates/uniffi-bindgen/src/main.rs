use anyhow::Context;
use camino::Utf8Path;
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
        #[arg(long)]
        crate_filter: Option<String>,
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
            library,
            language,
            config,
            out_dir,
            no_format,
            crate_filter,
        } => match language.as_str() {
            "csharp" => {
                use uniffi_bindgen::cargo_metadata::CrateConfigSupplier;
                use uniffi_bindgen::library_mode::generate_bindings;

                let config_supplier = {
                    let cmd = ::cargo_metadata::MetadataCommand::new();
                    let metadata = cmd.exec().unwrap();
                    CrateConfigSupplier::from(metadata)
                };

                let library: &Utf8Path = library.as_path().try_into().unwrap();
                let out_dir: &Utf8Path = out_dir.as_path().try_into().unwrap();
                let config: Option<&Utf8Path> = config.as_ref().map(|p| {
                    let r: &Utf8Path = p.as_path().try_into().unwrap();
                    r
                });
                let crate_name: Option<String> = crate_filter.map(|s| s.to_string());

                generate_bindings(
                    library,
                    crate_name,
                    &uniffi_bindgen_cs::BindingGenerator {
                        try_format_code: !no_format,
                    },
                    &config_supplier,
                    config,
                    out_dir,
                    !no_format,
                )
                .context("Failed to generate C# bindings")
                .expect("Failed to generate C# bindings");
            }
            _ => {
                uniffi::uniffi_bindgen_main();
            }
        },
    }
}
