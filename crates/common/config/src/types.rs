use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub agent: AgentConfig,
    pub log: LogConfig,
    pub push: PushConfig,
    pub checks: Vec<Check>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct PushConfig {
    pub url: String,
    pub token: String,
    pub default_status: String,
    pub default_msg: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Check {
    Http {
        id: String,
        url: String,
        #[serde(default = "default_method")]
        method: String,
        expected_status: Vec<u16>,
        timeout: u64,
        #[serde(default = "default_true")]
        follow_redirects: bool,
    },
    Https {
        id: String,
        url: String,
        #[serde(default = "default_method")]
        method: String,
        expected_status: Vec<u16>,
        timeout: u64,
        #[serde(default)]
        insecure_skip_verify: bool,
    },
    Tcp {
        id: String,
        host: String,
        port: u16,
        timeout: u64,
    },
    Ping {
        id: String,
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
