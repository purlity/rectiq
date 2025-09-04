// rectiq-cli/src/commands/keystore.rs

use crate::keystore::{self, KeyStoreError};
use clap::{Args, Subcommand};
use zeroize::Zeroizing;

/// Manage Rectiq secrets in the platform keystore.
#[derive(Debug, Args)]
pub struct KeystoreArgs {
    #[command(subcommand)]
    pub action: KeystoreAction,
}

#[derive(Debug, Subcommand)]
pub enum KeystoreAction {
    /// Get a secret by id. Prints masked by default; use --show to print full value.
    Get {
        /// Fully-qualified key id (e.g., `rectiq:prod:default:api_key`)
        id: String,
        /// Print full, unmasked value (use with care)
        #[arg(long)]
        show: bool,
    },
    /// Set a secret by id.
    Set {
        /// Fully-qualified key id (e.g., `rectiq:prod:default:api_key`)
        id: String,
        /// Secret value
        value: String,
    },
    /// Delete a secret by id.
    Delete {
        /// Fully-qualified key id (e.g., `rectiq:prod:default:api_key`)
        id: String,
    },
    /// Check if a secret exists (exit code 0 if present; prints a status line).
    Present {
        /// Fully-qualified key id (e.g., `rectiq:prod:default:api_key`)
        id: String,
    },
}

/// Dispatch the keystore subcommands.
///
/// # Errors
/// Returns an error if the underlying keystore operation fails.
pub fn dispatch(args: KeystoreArgs) -> anyhow::Result<()> {
    // Use the default store, which prefers the platform keystore and falls back to memory.
    let store = keystore::default_store();

    match args.action {
        KeystoreAction::Get { id, show } => match store.get(&id) {
            Ok(secret) => {
                let secret = Zeroizing::new(secret);
                if show {
                    println!("{}", &*secret);
                } else {
                    let s = &*secret;
                    let masked = if s.len() <= 8 {
                        "***".to_string()
                    } else {
                        format!("{}…{}", &s[..4], &s[s.len() - 4..])
                    };
                    println!("present: {masked}");
                }
            }
            Err(KeyStoreError::NotFound) => {
                println!("absent");
            }
            Err(e) => return Err(e.into()),
        },
        KeystoreAction::Set { id, value } => {
            // Minimize lifetime of the sensitive value in memory.
            let val = Zeroizing::new(value);
            store.set(&id, &val)?;
            println!("set: {id}");
        }
        KeystoreAction::Delete { id } => {
            store.delete(&id)?;
            println!("deleted: {id}");
        }
        KeystoreAction::Present { id } => {
            let ok = store.get(&id).is_ok();
            println!("{}: {}", id, if ok { "present" } else { "absent" });
        }
    }

    Ok(())
}
