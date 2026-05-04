use anyhow::{Context, Result};
use app_config::PushConfig;

use crate::checks::CheckRun;

const MAX_MSG_LEN: usize = 1500;

/// Sends a heartbeat to the Uptime Kuma push URL for this monitor only.
pub async fn send_push_for_check(
    push: &PushConfig,
    push_token: &str,
    run: &CheckRun,
) -> Result<()> {
    let status = if run.ok {
        normalize_status(&push.default_status, true)
    } else {
        "down".to_string()
    };

    let msg = if run.ok {
        push.default_msg.clone()
    } else {
        truncate(run.detail.clone(), MAX_MSG_LEN)
    };

    let endpoint = build_push_endpoint(push, push_token);
    let client = reqwest::Client::new();

    let mut req = client
        .get(&endpoint)
        .query(&[("status", status.as_str()), ("msg", msg.as_str())]);

    if run.ok && run.latency_ms > 0 {
        let ping = run.latency_ms.to_string();
        req = req.query(&[("ping", ping.as_str())]);
    }

    let resp = req
        .send()
        .await
        .with_context(|| format!("push GET {} (check {})", endpoint, run.id))?;
    if !resp.status().is_success() {
        anyhow::bail!(
            "push failed for check {} with HTTP {}",
            run.id,
            resp.status()
        );
    }

    Ok(())
}

fn build_push_endpoint(push: &PushConfig, push_token: &str) -> String {
    format!(
        "{}/{}",
        push.url.trim_end_matches('/'),
        push_token.trim_start_matches('/')
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

fn truncate(s: String, max: usize) -> String {
    if s.len() <= max {
        return s;
    }
    format!("{}…", &s[..max.saturating_sub(1)])
}
