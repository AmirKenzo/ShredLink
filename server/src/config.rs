use anyhow::{Context, Result};
use std::env;

/// Dev-only key (32 zero bytes, base64). Do not use in production.
const DEV_ENCRYPTION_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub encryption_key_base64: String,
    pub create_rate_limit_per_minute: u32,
    pub max_text_size_bytes: usize,
    pub cleanup_interval_secs: u64,
    pub base_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .context("PORT must be a number")?;
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:data/shredlink.db?mode=rwc".to_string());
        let encryption_key_base64 = env::var("ENCRYPTION_KEY").unwrap_or_else(|_| {
            tracing::warn!(
                "ENCRYPTION_KEY not set; using dev key. Set ENCRYPTION_KEY in .env for production (e.g. openssl rand -base64 32)"
            );
            DEV_ENCRYPTION_KEY.to_string()
        });
        let create_rate_limit_per_minute = env::var("CREATE_RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);
        let max_text_size_bytes = env::var("MAX_TEXT_SIZE_BYTES")
            .unwrap_or_else(|_| "100000".to_string())
            .parse()
            .unwrap_or(100_000);
        let cleanup_interval_secs = env::var("CLEANUP_INTERVAL_SECS")
            .unwrap_or_else(|_| "600".to_string())
            .parse()
            .unwrap_or(600);
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

        Ok(Self {
            host,
            port,
            database_url,
            encryption_key_base64,
            create_rate_limit_per_minute,
            max_text_size_bytes,
            cleanup_interval_secs,
            base_url,
        })
    }
}
