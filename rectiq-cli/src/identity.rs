#![forbid(unsafe_code)]

use crate::config::CliConfig;
use crate::keystore::{key_id, default_store};
use anyhow::{anyhow, Context, Result};
use dpop::{DeviceKey, EcJwk};
use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct DeviceStartReq<'a> {
    email: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceStartResp {
    #[serde(default)]
    verification_uri: Option<String>,
    #[serde(default)]
    user_code: Option<String>,
    device_code: String,
    #[serde(default = "default_poll_interval")]
    interval: u64,
}

const fn default_poll_interval() -> u64 {
    3
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceCompleteReq<'a> {
    device_code: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceCompleteResp {
    refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenReq<'a> {
    refresh_token: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResp {
    access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WhoamiResp {
    org: String,
    user: String,
    #[serde(default)]
    plan: Option<String>,
    #[serde(default)]
    limits: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DevicesRegisterReq {
    jwk: EcJwk,
}

#[derive(Debug, Serialize, Deserialize)]
struct DevicesRegisterResp {
    device_id: String,
    #[serde(default)]
    jkt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeysReq<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    bind_device_id: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeysResp {
    secret_full: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceRecord {
    device_id: String,
    jkt: String,
    private_jwk: EcJwk,
}

pub struct IdentityClient {
    cfg: CliConfig,
    http: Client,
}

impl IdentityClient {
    /// Create a new identity client bound to the given config.
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be constructed.
    pub fn new(cfg: CliConfig) -> Result<Self> {
        let http = Client::builder().timeout(cfg.http_timeout).build()?;
        Ok(Self { cfg, http })
    }

    fn bearer(token: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            reqwest::header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
        h
    }

    fn post_json<T: Serialize + ?Sized>(&self, url: &str, body: &T) -> Result<Response> {
        Ok(self.http.post(url).json(body).send()?)
    }

    /// Run the device onboarding flow for the specified email.
    ///
    /// # Errors
    /// Returns an error if any network call fails or secret persistence fails.
    pub fn onboard(&self, email: &str) -> Result<()> {
        // 1) Generate device keypair (in-memory)
        let device = DeviceKey::generate()?;

        // 2) Device start
        let start = DeviceStartReq { email };
        let start_url = self.cfg.identity_device_start_url();
        let resp: DeviceStartResp = self.post_json(&start_url, &start)?.json()?;
        if let Some(uri) = resp.verification_uri.as_ref() {
            eprintln!("Open to verify: {uri}");
        }
        if let Some(code) = resp.user_code.as_ref() {
            eprintln!("Enter code: {code}");
        }

        // 3) Poll device complete
        let complete_url = self.cfg.identity_device_complete_url();
        let start_time = std::time::Instant::now();
        let max_wait = Duration::from_secs(180);
        let refresh_token: String = loop {
            if start_time.elapsed() > max_wait {
                return Err(anyhow!("timeout waiting for device verification"));
            }
            let req = DeviceCompleteReq {
                device_code: &resp.device_code,
            };
            let r = self.post_json(&complete_url, &req)?;
            if r.status().is_success() {
                let body: DeviceCompleteResp = r.json()?;
                break body.refresh_token;
            }
            sleep(Duration::from_secs(resp.interval.max(1)));
        };

        // 4) Obtain access token and whoami
        let token_url = self.cfg.auth_token_url();
        let t: TokenResp = self
            .post_json(
                &token_url,
                &TokenReq {
                    refresh_token: &refresh_token,
                },
            )?
            .json()?;
        let whoami: WhoamiResp = self
            .http
            .get(self.cfg.whoami_url())
            .headers(Self::bearer(&t.access_token))
            .send()?
            .json()?;
        let who = format!("{}/{}", whoami.org, whoami.user);

        // 5) Register device
        let reg: DevicesRegisterResp = self
            .post_json(
                &self.cfg.devices_register_url(),
                &DevicesRegisterReq {
                    jwk: device.public_jwk().clone(),
                },
            )?
            .json()?;
        let device_id = reg.device_id;
        let jkt = reg.jkt.unwrap_or_else(|| device.public_jwk().thumbprint());

        // 6) Mint API key bound to device
        let k: KeysResp = self
            .post_json(
                &self.cfg.keys_url(),
                &KeysReq {
                    bind_device_id: Some(&device_id),
                },
            )?
            .json()?;

        // 7) Persist secrets in keystore
        let store = default_store();
        let env = &self.cfg.profile;
        let refresh_key = key_id(env, &who, "refresh");
        let api_key_key = key_id(env, &who, "api_key");
        let device_key = key_id(env, &who, "device");
        let device_rec = DeviceRecord {
            device_id,
            jkt,
            private_jwk: device.private_jwk(),
        };
        // We cannot get SigningKey from DeviceKey without exposing; reconstruct from private_jwk
        // Workaround: serialize private via from_sk above using a fresh SigningKey
        let rec_json = serde_json::to_string(&device_rec)?;
        store.set(&refresh_key, &refresh_token)?;
        store.set(&api_key_key, &k.secret_full)?;
        store.set(&device_key, &rec_json)?;

        // 8) Ping with DPoP
        self.ping_with_device(&who)?;
        eprintln!("Onboard complete for {who}");
        Ok(())
    }

    /// Mint a new API key for the given `org/user` identity, binding it to the stored device.
    ///
    /// # Errors
    /// Returns an error if secrets cannot be loaded or the key mint call fails.
    pub fn request_api_key(&self, who: &str) -> Result<()> {
        let store = default_store();
        let env = &self.cfg.profile;
        let refresh_key = key_id(env, who, "refresh");
        let refresh_token = store
            .get(&refresh_key)
            .map_err(|_| anyhow!("missing refresh token; run 'rectiq onboard'"))?;
        // optional: ensure device_id exists
        let device_key = key_id(env, who, "device");
        let rec_json = store
            .get(&device_key)
            .map_err(|_| anyhow!("missing device record; run 'rectiq onboard'"))?;
        let device_rec: DeviceRecord =
            serde_json::from_str(&rec_json).context("decode device record")?;
        let _ = refresh_token; // currently not used by server for key minting if device binding is sufficient
        let k: KeysResp = self
            .post_json(
                &self.cfg.keys_url(),
                &KeysReq {
                    bind_device_id: Some(&device_rec.device_id),
                },
            )?
            .json()?;
        let api_key_key = key_id(env, who, "api_key");
        store.set(&api_key_key, &k.secret_full)?;
        eprintln!("New API key minted and stored for {who}");
        Ok(())
    }

    /// Print identity properties for the given `org/user`. Falls back to refresh token if API key is missing.
    ///
    /// # Errors
    /// Returns an error if no credentials are found or requests fail.
    pub fn whoami(&self, who: Option<&str>) -> Result<()> {
        let store = default_store();
        let env = &self.cfg.profile;
        // Prefer API key when available
        if let Some(w) = who {
            let api_key_key = key_id(env, w, "api_key");
            if let Ok(api) = store.get(&api_key_key) {
                let json: WhoamiResp = self
                    .http
                    .get(self.cfg.whoami_url())
                    .headers(Self::bearer(&api))
                    .send()?
                    .json()?;
                println!("org: {}", json.org);
                println!("user: {}", json.user);
                if let Some(plan) = json.plan {
                    println!("plan: {plan}");
                }
                if let Some(limits) = json.limits {
                    println!("limits: {limits}");
                }
                return Ok(());
            }
        }
        // Otherwise try refresh->access
        if let Some(w) = who {
            let refresh_key = key_id(env, w, "refresh");
            if let Ok(refresh) = store.get(&refresh_key) {
                let t: TokenResp = self
                    .post_json(
                        &self.cfg.auth_token_url(),
                        &TokenReq {
                            refresh_token: &refresh,
                        },
                    )?
                    .json()?;
                let json: WhoamiResp = self
                    .http
                    .get(self.cfg.whoami_url())
                    .headers(Self::bearer(&t.access_token))
                    .send()?
                    .json()?;
                println!("org: {}", json.org);
                println!("user: {}", json.user);
                if let Some(plan) = json.plan {
                    println!("plan: {plan}");
                }
                if let Some(limits) = json.limits {
                    println!("limits: {limits}");
                }
                return Ok(());
            }
        }
        Err(anyhow!(
            "no credentials found; pass --who or run 'rectiq onboard'"
        ))
    }

    /// Perform a `DPoP`-protected ping using the device key for `org/user`.
    ///
    /// # Errors
    /// Returns an error if loading keys fails or the server returns non-success.
    pub fn ping(&self, who: &str) -> Result<()> {
        self.ping_with_device(who)
    }

    fn ping_with_device(&self, who: &str) -> Result<()> {
        // Load device record
        let store = default_store();
        let env = &self.cfg.profile;
        let device_key = key_id(env, who, "device");
        let rec_json = store
            .get(&device_key)
            .map_err(|_| anyhow!("missing device record; run 'rectiq onboard'"))?;
        let device_rec: DeviceRecord =
            serde_json::from_str(&rec_json).context("decode device record")?;
        let dev = dpop::DeviceKey::from_private_jwk(&device_rec.private_jwk)?;
        let url = self.cfg.ping_url();

        // Initial request without nonce
        let proof = dev.dpop_proof("GET", &url, None)?;
        let mut http_req = self.http.get(&url);
        http_req = http_req.header("DPoP", proof);
        let r = http_req.send()?;

        if r.status().is_success() {
            println!("200 OK");
            return Ok(());
        }
        // Retry with nonce if provided
        if let Some(nonce) = r.headers().get("DPoP-Nonce").and_then(|v| v.to_str().ok()) {
            let proof2 = dev.dpop_proof("GET", &url, Some(nonce))?;
            let r2 = self.http.get(&url).header("DPoP", proof2).send()?;
            if r2.status().is_success() {
                println!("200 OK");
                return Ok(());
            }
            return Err(anyhow!("ping failed: {}", r2.status()));
        }
        Err(anyhow!("ping failed: {}", r.status()))
    }
}
