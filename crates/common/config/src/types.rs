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
}

/// One ICMP target: identity, dedicated UK push token, and ping settings.
#[derive(Debug, Clone, Deserialize)]
pub struct CheckEntry {
    pub id: String,
    /// Uptime Kuma push token for this monitor (from the Push monitor URL).
    pub push_token: String,
    pub host: String,
    pub timeout: u64,
}
