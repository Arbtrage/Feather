use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_lease_duration_ms")]
    pub default_lease_duration_ms: u64,
    #[serde(default = "default_max_payload")]
    pub max_payload_bytes: usize,
    #[serde(default = "default_sweep_interval")]
    pub lease_sweep_interval_ms: u64,
    #[serde(default = "default_history_limit")]
    pub recent_history_limit: usize,
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_ms: u64,
    #[serde(default = "default_offline_threshold")]
    pub offline_threshold_ms: u64,
    #[serde(default = "default_max_renewals")]
    pub max_lease_renewals: u32,
    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,
    pub server: ServerConfig,
    pub storage: StorageConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub grpc_addr: String,
    pub http_addr: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub redis_url: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ObservabilityConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_log_format")]
    pub log_format: String,
}

fn default_lease_duration_ms() -> u64 {
    30_000
}
fn default_max_payload() -> usize {
    262_144
}
fn default_sweep_interval() -> u64 {
    1_000
}
fn default_history_limit() -> usize {
    10_000
}
fn default_heartbeat_interval() -> u64 {
    10_000
}
fn default_offline_threshold() -> u64 {
    30_000
}
fn default_max_renewals() -> u32 {
    100
}
fn default_cors_origins() -> Vec<String> {
    vec![
        "http://localhost:3000".into(),
        "http://localhost:3001".into(),
    ]
}
fn default_log_level() -> String {
    "info".into()
}
fn default_log_format() -> String {
    "json".into()
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let path = Path::new("config/default.toml");
        let mut cfg: AppConfig = if path.exists() {
            let content = std::fs::read_to_string(path)?;
            toml::from_str(&content)?
        } else {
            AppConfig {
                default_lease_duration_ms: default_lease_duration_ms(),
                max_payload_bytes: default_max_payload(),
                lease_sweep_interval_ms: default_sweep_interval(),
                recent_history_limit: default_history_limit(),
                heartbeat_interval_ms: default_heartbeat_interval(),
                offline_threshold_ms: default_offline_threshold(),
                max_lease_renewals: default_max_renewals(),
                cors_origins: default_cors_origins(),
                server: ServerConfig {
                    grpc_addr: "0.0.0.0:50051".into(),
                    http_addr: "0.0.0.0:8080".into(),
                },
                storage: StorageConfig {
                    redis_url: "redis://127.0.0.1:6379".into(),
                    namespace: "default".into(),
                },
                observability: ObservabilityConfig::default(),
            }
        };

        if let Ok(v) = std::env::var("FEATHER_GRPC_ADDR") {
            cfg.server.grpc_addr = v;
        }
        if let Ok(v) = std::env::var("FEATHER_HTTP_ADDR") {
            cfg.server.http_addr = v;
        }
        if let Ok(v) = std::env::var("FEATHER_REDIS_URL") {
            cfg.storage.redis_url = v;
        }
        if let Ok(v) = std::env::var("FEATHER_NAMESPACE") {
            cfg.storage.namespace = v;
        }
        if let Ok(v) = std::env::var("FEATHER_LEASE_MS") {
            cfg.default_lease_duration_ms = v.parse()?;
        }
        if let Ok(v) = std::env::var("FEATHER_LOG") {
            cfg.observability.log_level = v;
        }
        if let Ok(v) = std::env::var("FEATHER_CORS_ORIGINS") {
            cfg.cors_origins = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        Ok(cfg)
    }
}
