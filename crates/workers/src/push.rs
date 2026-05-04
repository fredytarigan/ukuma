use anyhow::{Context, Result};
use app_config::{AppConfig, PushConfig};

use crate::checks::CheckRun;

const MAX_MSG_LEN: usize = 1500;

pub async fn send_push(config: &AppConfig, runs: &[CheckRun]) -> Result<()> {
    let all_ok = runs.iter().all(|r| r.ok);
    let status = if all_ok {
        normalize_status(&config.push.default_status, true)
    } else {
        "down".to_string()
    };

    let msg = if all_ok {
        config.push.default_msg.clone()
    } else {
        truncate(summarize_failures(runs), MAX_MSG_LEN)
    };

    let ping_ms = runs
        .iter()
        .filter(|r| r.ok)
        .map(|r| r.latency_ms)
        .max()
        .unwrap_or(0);

    let endpoint = build_push_endpoint(&config.push);
    let client = reqwest::Client::new();

    let mut req = client
        .get(&endpoint)
        .query(&[("status", status.as_str()), ("msg", msg.as_str())]);

    if ping_ms > 0 {
        let ping = ping_ms.to_string();
        req = req.query(&[("ping", ping.as_str())]);
    }

    let resp = req.send().await.with_context(|| format!("push GET {endpoint}"))?;
    if !resp.status().is_success() {
        anyhow::bail!("push failed with HTTP {}", resp.status());
    }

    Ok(())
}

fn build_push_endpoint(push: &PushConfig) -> String {
    format!(
        "{}/{}",
        push.url.trim_end_matches('/'),
        push.token.trim_start_matches('/')
    )
}

fn normalize_status(configured: &str, success: bool) -> String {
    let t = configured.trim().to_ascii_lowercase();
    if t == "up" || t == "down" {
        return t;
    }
    if success {
        "up".to_string()
    } else {
        "down".to_string()
    }
}

fn summarize_failures(runs: &[CheckRun]) -> String {
    runs
        .iter()
        .filter(|r| !r.ok)
        .map(|r| format!("{}: {}", r.id, r.detail))
        .collect::<Vec<_>>()
        .join("; ")
}

fn truncate(s: String, max: usize) -> String {
    if s.len() <= max {
        return s;
    }
    format!("{}…", &s[..max.saturating_sub(1)])
}
