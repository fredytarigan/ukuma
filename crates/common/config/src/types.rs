use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub agent: AgentConfig,
    pub log: LogConfig,
    pub push: PushConfig,
    pub checks: Vec<CheckEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    /// Seconds between check runs / heartbeats.
    pub interval: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    pub level: String,
}

/// Shared Uptime Kuma push API base (no monitor token). Each check supplies its own token.
#[derive(Debug, Clone, Deserialize)]
pub struct PushConfig {
    /// e.g. `https://your-kuma.example.com/api/push`
    pub url: String,
    pub default_status: String,
    pub default_msg: String,
}

/// One logical monitor: identity, dedicated UK push token, and probe settings.
#[derive(Debug, Clone, Deserialize)]
pub struct CheckEntry {
    pub id: String,
    /// Uptime Kuma push token for this monitor (from the Push monitor URL).
    pub push_token: String,
    #[serde(flatten)]
    pub probe: CheckProbe,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CheckProbe {
    Http {
        url: String,
        #[serde(default = "default_method")]
        method: String,
        expected_status: Vec<u16>,
        timeout: u64,
        #[serde(default = "default_true")]
        follow_redirects: bool,
    },
    Https {
        url: String,
        #[serde(default = "default_method")]
        method: String,
        expected_status: Vec<u16>,
        timeout: u64,
        #[serde(default)]
        insecure_skip_verify: bool,
    },
    Tcp {
        host: String,
        port: u16,
        timeout: u64,
    },
    Ping {
        host: String,
        timeout: u64,
    },
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_true() -> bool {
    true
}
