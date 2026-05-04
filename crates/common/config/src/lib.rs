mod config;
mod types;

pub use config::{read_from_path, read_from_str, LoadError};
pub use types::{AgentConfig, AppConfig, Check, PushConfig};

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
        assert_eq!(cfg.push.token, "your_token_here");
        assert_eq!(cfg.checks.len(), 4);

        match &cfg.checks[0] {
            Check::Http {
                id,
                url,
                method,
                expected_status,
                timeout,
                follow_redirects,
            } => {
                assert_eq!(id, "example-http");
                assert!(url.contains("generate_204"));
                assert_eq!(method, "GET");
                assert_eq!(expected_status, &[200]);
                assert_eq!(*timeout, 5000);
                assert!(*follow_redirects);
            }
            _ => panic!("first check should be http"),
        }
    }
}
