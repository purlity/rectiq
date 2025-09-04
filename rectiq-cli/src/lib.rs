// rectiq-cli/src/lib.rs
pub mod commands;
pub mod controller;
pub mod divine;
pub mod fix_executor;
pub mod local_coordinator;
pub mod pipeline;
pub mod remote;
pub mod keystore;
pub mod security {
    pub mod aad;
    pub mod http;
    pub mod redact;
}
pub mod sketches;
pub mod symbolset;
pub mod types;
pub mod utils;
pub mod config;
pub mod identity;
pub mod svc;

use crate::{
    commands::KeystoreArgs, config::is_dev, controller::masking::to_masked_owned_envelope,
    local_coordinator::LocalCoordinator,
};
use anyhow::{anyhow, Result};
use clap::Parser;
use rectiq_types::SketchNode;
use std::path::{Path, PathBuf};
use tracing::info;
use walkdir::WalkDir;
use zeroize::Zeroizing;
pub use sketches::{ShapeSketcher, SketchOrchestrator, TokenSketcher};

/// Scan JSON and return fully owned sketch envelopes.
pub fn scan(input: &str) -> Vec<SketchNode<'static>> {
    LocalCoordinator::new()
        .scan(input)
        .sketches
        .iter()
        .map(to_masked_owned_envelope)
        .collect()
}

/// Run full fixer pipeline by sending to backend API
///
/// # Errors
/// Returns an error when the input payload fails validation.
pub fn run(input: &str, user_id: &str) -> Result<String, String> {
    let mut coordinator = LocalCoordinator::new();
    coordinator.run_with_api(input, user_id)
}

/// Discover all files under `dir` recursively.
///
/// # Errors
/// Returns an error when the input payload fails validation.
pub fn discover_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    Ok(files)
}

#[derive(clap::Parser)]
#[command(name = "rectiq", version, author)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Scan a directory of JSON files
    Scan { dir: String },
    /// Sketch the issues in a JSON file
    Sketch {
        file: String,
        #[clap(long)]
        json: bool,
    },
    /// Fix a JSON file via the backend API
    Fix { file: String },
    /// Initialize a random ZK-compliant symbol set
    SymbolsetInit,
    /// View the current ZK-compliant symbol set
    SymbolsetView,
    /// Keystore management commands
    Keystore {
        /// Arguments to pass through to the keystore subcommand
        #[command(flatten)]
        args: KeystoreArgs,
    },
    /// First-time device onboarding and key mint
    Onboard {
        /// Email for magic-link/device flow
        #[clap(long)]
        email: String,
    },
    /// Display identity info for the active credentials
    Whoami {
        /// Identity scope 'org/user' that holds credentials
        #[clap(long)]
        who: Option<String>,
    },
    /// `DPoP` ping using device key
    Ping {
        /// Identity scope 'org/user' whose device to use
        #[clap(long)]
        who: String,
    },
    /// Mint a new API key bound to the existing device
    RequestApiKey {
        /// Identity scope 'org/user' to bind the key to
        #[clap(long)]
        who: String,
    },
    /// Service account management
    Svc {
        #[command(subcommand)]
        cmd: SvcCommands,
    },
}

#[derive(clap::Subcommand)]
enum SvcCommands {
    /// Create a service account
    Create {
        /// Service account name
        name: String,
        /// Comma-separated scopes (e.g., fix:write)
        #[clap(long, value_delimiter = ',')]
        scopes: Option<Vec<String>>,
    },
    /// Mint a short-lived token for a service account
    TokenMint {
        /// Service account name to mint for
        #[clap(long)]
        service: String,
        /// TTL (e.g., 15m, 3600)
        #[clap(long)]
        ttl: Option<String>,
    },
}

/// Run the CLI with an arbitrary iterator of arguments.
///
/// # Errors
/// Returns an error when the input payload fails validation.
pub fn run_with_args<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    // Tracing and panic hook are initialized in `main`.
    #[cfg(feature = "parallel")]
    info!("Rectiq running with Rayon parallelism enabled.");
    #[cfg(not(feature = "parallel"))]
    info!("Rectiq running in sequential (safe) mode.");

    let cli = Cli::parse_from(args);

    let ks: Box<dyn keystore::KeyStore> = if cfg!(test) || is_dev() {
        Box::new(keystore::memory::MemoryKeyStore::default())
    } else {
        keystore::default_store()
    };

    // Prefer keystore first; env only seeds it (best effort)
    let license_env = std::env::var("RECTIQ_LICENSE_KEY").ok().map(Zeroizing::new);
    let license = ks.get("license:default").map_or_else(
        |_| {
            if let Some(ref val) = license_env {
                let _ = ks.set("license:default", val);
            }
            license_env.clone()
        },
        |v| Some(Zeroizing::new(v)),
    );

    match cli.command {
        Commands::Scan { dir } => {
            commands::scan_mode(&dir)?;
        }
        Commands::Sketch { file, .. } => {
            commands::sketch_mode(&file);
        }
        Commands::Fix { file } => {
            let license = license
                .ok_or_else(|| {
                    anyhow!(
                        "401 unauthorized: missing Rectiq license key. Set RECTIQ_LICENSE_KEY env var or login"
                    )
                })?;
            commands::fix_mode(&file, &license);
            if license_env.is_some() {
                // Persist the license so future runs can fall back to the keystore.
                ks.set("license:default", &license)?;
            }
        }
        Commands::SymbolsetInit => commands::init_symbolset(),
        Commands::SymbolsetView => commands::view_symbolset(),
        Commands::Keystore { args } => {
            commands::dispatch(args)?;
        }
        Commands::Onboard { email } => {
            let cfg = config::CliConfig::default();
            let id = identity::IdentityClient::new(cfg)?;
            id.onboard(&email)?;
        }
        Commands::Whoami { who } => {
            let cfg = config::CliConfig::default();
            let id = identity::IdentityClient::new(cfg)?;
            id.whoami(who.as_deref())?;
        }
        Commands::Ping { who } => {
            let cfg = config::CliConfig::default();
            let id = identity::IdentityClient::new(cfg)?;
            id.ping(&who)?;
        }
        Commands::RequestApiKey { who } => {
            let cfg = config::CliConfig::default();
            let id = identity::IdentityClient::new(cfg)?;
            id.request_api_key(&who)?;
        }
        Commands::Svc { cmd } => {
            let cfg = config::CliConfig::default();
            let sc = svc::SvcClient::new(cfg)?;
            match cmd {
                SvcCommands::Create { name, scopes } => {
                    let id = sc.create(&name, scopes.unwrap_or_default())?;
                    println!("service_id: {}", id);
                }
                SvcCommands::TokenMint { service, ttl } => {
                    let res = sc.mint_token(&service, ttl.as_deref())?;
                    println!("access_token: {}", res.access_token);
                    println!("exp: {}", res.exp);
                }
            }
        }
    }
    Ok(())
}
