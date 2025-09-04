#![forbid(unsafe_code)]

use crate::config::CliConfig;
use crate::keystore::default_store;
use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SvcCreateReq<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    scopes: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SvcCreateRes {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SvcTokenReq<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SvcTokenRes {
    pub access_token: String,
    pub exp: i64,
}

pub struct SvcClient {
    cfg: CliConfig,
    http: Client,
}

impl SvcClient {
    pub fn new(cfg: CliConfig) -> Result<Self> {
        let http = Client::builder().timeout(cfg.http_timeout).build()?;
        Ok(Self { cfg, http })
    }

    fn bearer(t: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            reqwest::header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {t}")).unwrap(),
        );
        h
    }

    pub fn create(&self, name: &str, scopes: Vec<String>) -> Result<String> {
        // Exchange API key -> access token
        let api_key = std::env::var("RECTIQ_API_KEY")
            .map_err(|_| anyhow!("missing RECTIQ_API_KEY for admin"))?;
        let t_url = self.cfg.auth_token_url();
        let r = self
            .http
            .post(t_url)
            .header("Authorization", format!("RectiqKey {}", api_key))
            .send()?;
        if !r.status().is_success() {
            return Err(anyhow!("token exchange failed: {}", r.status()));
        }
        let j: serde_json::Value = r.json()?;
        let access = j
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("missing access_token"))?;
        let url = self.cfg.svc_url();
        let body = SvcCreateReq {
            name,
            scopes: if scopes.is_empty() {
                None
            } else {
                Some(scopes)
            },
        };
        let r = self
            .http
            .post(url)
            .headers(Self::bearer(access))
            .json(&body)
            .send()?;
        if !r.status().is_success() {
            return Err(anyhow!("svc create failed: {}", r.status()));
        }
        let resp: SvcCreateRes = r.json()?;
        // persist id under profile namespace for convenience
        let ks = default_store();
        let key = format!("svc:{}:id", name);
        ks.set(&key, &resp.id)?;
        Ok(resp.id)
    }

    /// Mint a token for service `{name}`. Requires RECTIQ_API_KEY env for admin.
    pub fn mint_token(&self, name: &str, ttl: Option<&str>) -> Result<SvcTokenRes> {
        // Resolve id from keystore
        let ks = default_store();
        let key = format!("svc:{}:id", name);
        let id = ks
            .get(&key)
            .map_err(|_| anyhow!("service id not found; run 'rectiq svc create' first"))?;
        // Exchange API key -> access token
        let api_key = std::env::var("RECTIQ_API_KEY")
            .map_err(|_| anyhow!("missing RECTIQ_API_KEY for admin"))?;
        let t_url = self.cfg.auth_token_url();
        let r = self
            .http
            .post(t_url)
            .header("Authorization", format!("RectiqKey {}", api_key))
            .send()?;
        if !r.status().is_success() {
            return Err(anyhow!("token exchange failed: {}", r.status()));
        }
        let j: serde_json::Value = r.json()?;
        let access = j
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("missing access_token"))?;
        // Mint service token
        let url = self.cfg.svc_token_url(&id);
        let r = self
            .http
            .post(url)
            .headers(Self::bearer(access))
            .json(&SvcTokenReq { ttl })
            .send()?;
        if !r.status().is_success() {
            return Err(anyhow!("svc token mint failed: {}", r.status()));
        }
        Ok(r.json()?)
    }
}
