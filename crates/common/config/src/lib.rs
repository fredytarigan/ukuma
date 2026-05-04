mod config;
mod types;

pub use config::{read_from_path, read_from_str, LoadError};
pub use types::{AgentConfig, AppConfig, CheckEntry, PushConfig};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn example_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../config/config.yaml.example")
    }

    #[test]
    fn reads_example_yaml() {
        let cfg = read_from_path(example_path()).expect("example config should parse");
        assert_eq!(cfg.agent.interval, 60);
        assert_eq!(cfg.push.url, "https://push.ukuma.io/api/push");
        assert_eq!(cfg.checks.len(), 2);

        let first = &cfg.checks[0];
        assert_eq!(first.id, "example-ping-a");
        assert_eq!(first.push_token, "push_token_a");
        assert_eq!(first.host, "127.0.0.1");
        assert_eq!(first.timeout, 2000);
    }
}
