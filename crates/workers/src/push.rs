use anyhow::{Context, Result};
use app_config::PushConfig;
use reqwest::Url;

use crate::checks::CheckRun;

const MAX_MSG_LEN: usize = 1500;

/// Sends a heartbeat to the Uptime Kuma push URL for this monitor only.
///
/// Query shape: `?status=up|down&msg=...&ping=<latency_ms>` (UK uses `ping` for the reported latency).
pub async fn send_push_for_check(
    push: &PushConfig,
    push_token: &str,
    run: &CheckRun,
) -> Result<()> {
    let (status, msg) = if run.ok {
        ("up", "OK".to_string())
    } else {
        ("down", truncate(run.detail.clone(), MAX_MSG_LEN))
    };

    let ping_ms = run.latency_ms.to_string();

    let endpoint = build_push_endpoint(push, push_token);
    let mut final_url = Url::parse(&endpoint)
        .with_context(|| format!("invalid push endpoint URL: {endpoint}"))?;
    {
        let mut q = final_url.query_pairs_mut();
        q.append_pair("status", status);
        q.append_pair("msg", msg.as_str());
        q.append_pair("ping", ping_ms.as_str());
    }

    let full_url = final_url.as_str().to_string();
    let check_id = run.id.clone();
    println!("[debug] uptime kuma push (check id={check_id}): {full_url}");

    let client = reqwest::Client::new();
    let resp = client
        .get(final_url)
        .send()
        .await
        .with_context(move || format!("push GET {full_url} (check {check_id})"))?;
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

fn truncate(s: String, max: usize) -> String {
    if s.len() <= max {
        return s;
    }
    format!("{}…", &s[..max.saturating_sub(1)])
}
