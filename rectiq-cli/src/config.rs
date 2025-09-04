// rectiq-cli/src/config.rs
use std::{env, fs, path::PathBuf, time::Duration};

#[derive(Debug, Clone)]
pub struct CliConfig {
    /// Base for Rectiq API (fix, reveal, etc.)
    pub api_base: String,
    /// Connect + read timeout for HTTP clients
    pub http_timeout: Duration,
    /// Inferred development mode flag (captured at construction time)
    is_dev: bool,
    /// Profile namespace for keystore entries
    pub profile: String,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self::from_env_or_infer()
    }
}

impl CliConfig {
    /// Build config from env if provided, otherwise infer dev/prod.
    ///
    /// Heuristics:
    /// - If `RECTIQ_ENV` is set to `dev`/`development`/`local` → dev mode.
    /// - If `RECTIQ_ENV` is set to `prod`/`production` → prod mode.
    /// - Otherwise, fall back to `cfg!(debug_assertions)` for dev builds.
    /// - Env vars still override bases/timeouts when present.
    #[must_use]
    pub fn from_env_or_infer() -> Self {
        let env_mode = env::var("RECTIQ_ENV").ok();
        let is_dev = match env_mode.as_deref() {
            Some("dev" | "development" | "local") => true,
            Some("prod" | "production") => false,
            _ => cfg!(debug_assertions),
        };

        // Try to load from file config first
        let mut file_api_base: Option<String> = None;
        let mut file_profile: Option<String> = None;
        if let Some(path) = config_file_path()
            && let Ok(raw) = fs::read_to_string(&path)
            && let Ok(tbl) = raw.parse::<toml::Table>()
        {
            if let Some(v) = tbl.get("api_base").and_then(|v| v.as_str()) {
                file_api_base = Some(v.to_string());
            }
            if let Some(v) = tbl.get("profile").and_then(|v| v.as_str()) {
                file_profile = Some(v.to_string());
            }
        }

        // Bases: env overrides first, else file, else heuristic defaults
        let mut api_base = env::var("RECTIQ_API_BASE")
            .ok()
            .or(file_api_base)
            .unwrap_or_else(|| {
                if is_dev {
                    "http://127.0.0.1:8080/api".to_string()
                } else {
                    "https://api.rectiq.com".to_string()
                }
            });
        // normalize: strip single trailing slash to simplify join later
        api_base.truncate(api_base.trim_end_matches('/').len());

        // Timeout: env override, else 5s in dev, 15s in prod
        let http_timeout = env::var("RECTIQ_HTTP_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map_or_else(
                || {
                    if is_dev {
                        Duration::from_secs(5)
                    } else {
                        Duration::from_secs(15)
                    }
                },
                Duration::from_millis,
            );

        let profile = env::var("RECTIQ_PROFILE")
            .ok()
            .or(file_profile)
            .unwrap_or_else(|| "default".to_string());

        // Gentle warnings if insecure base is used outside dev
        if !is_dev && api_base.starts_with("http://") {
            eprintln!("[rectiq-cli] warning: using insecure API_BASE in prod mode: {api_base}");
        }

        Self {
            api_base,
            http_timeout,
            is_dev,
            profile,
        }
    }

    /// Quick check captured at construction time (no env re-read).
    #[must_use]
    pub const fn is_dev(&self) -> bool {
        self.is_dev
    }
}

impl CliConfig {
    /// Join a base with a path ensuring a single `/` boundary.
    fn join(base: &str, path: &str) -> String {
        debug_assert!(!base.ends_with('/'));
        path.strip_prefix('/')
            .map_or_else(|| format!("{base}/{path}"), |p| format!("{base}/{p}"))
    }

    /// POST /fix
    #[must_use]
    pub fn fix_url(&self) -> String {
        Self::join(&self.api_base, "/fix")
    }

    /// POST /reveal-step/{token}
    #[must_use]
    pub fn reveal_step_url(&self, token: &str) -> String {
        Self::join(&self.api_base, &format!("/reveal-step/{token}"))
    }

    /// GET /divine-key/{token}
    #[must_use]
    pub fn divine_key_url(&self, token: &str) -> String {
        Self::join(&self.api_base, &format!("/divine-key/{token}"))
    }

    /// POST /symbolset/init
    #[must_use]
    pub fn symbolset_init_url(&self) -> String {
        Self::join(&self.api_base, "/symbolset/init")
    }

    /// GET /symbolset/{id}
    #[must_use]
    pub fn symbolset_get_url(&self, id: &str) -> String {
        Self::join(&self.api_base, &format!("/symbolset/{id}"))
    }

    /// DELETE /symbolset/{id}
    #[must_use]
    pub fn symbolset_delete_url(&self, id: &str) -> String {
        Self::join(&self.api_base, &format!("/symbolset/{id}"))
    }

    // --- Identity / Auth endpoints ---
    #[must_use]
    pub fn identity_device_start_url(&self) -> String {
        Self::join(&self.api_base, "/v1/identity/device-start")
    }

    #[must_use]
    pub fn identity_device_complete_url(&self) -> String {
        Self::join(&self.api_base, "/v1/identity/device-complete")
    }

    #[must_use]
    pub fn devices_register_url(&self) -> String {
        Self::join(&self.api_base, "/v1/devices/register")
    }

    #[must_use]
    pub fn keys_url(&self) -> String {
        Self::join(&self.api_base, "/v1/keys")
    }

    #[must_use]
    pub fn auth_token_url(&self) -> String {
        Self::join(&self.api_base, "/v1/auth/token")
    }

    #[must_use]
    pub fn whoami_url(&self) -> String {
        Self::join(&self.api_base, "/v1/whoami")
    }

    #[must_use]
    pub fn ping_url(&self) -> String {
        Self::join(&self.api_base, "/v1/ping")
    }

    // --- Service accounts ---
    #[must_use]
    pub fn svc_url(&self) -> String {
        Self::join(&self.api_base, "/v1/svc")
    }

    #[must_use]
    pub fn svc_token_url(&self, svc_id: &str) -> String {
        Self::join(&self.api_base, &format!("/v1/svc/{svc_id}/token"))
    }

    // --- OIDC federation ---
    #[must_use]
    pub fn oidc_exchange_url(&self) -> String {
        Self::join(&self.api_base, "/v1/oidc/exchange")
    }
}

/// Determine if the CLI should run in development mode.
#[must_use]
pub fn is_dev() -> bool {
    match env::var("RECTIQ_ENV").ok().as_deref() {
        Some("dev" | "development" | "local") => true,
        Some("prod" | "production") => false,
        _ => cfg!(debug_assertions),
    }
}

/// Determine if the CLI is running in a test context.
#[must_use]
pub fn is_test() -> bool {
    matches!(
        env::var("RECTIQ_ENV").ok().as_deref(),
        Some("test" | "testing")
    ) || cfg!(test)
}

/// Resolve config file path.
fn config_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("rectiq").join("config.toml"))
}
