use std::net::IpAddr;
use std::time::{Duration, Instant};

use app_config::CheckEntry;
use tokio::net::lookup_host;

#[derive(Debug, Clone)]
pub struct CheckRun {
    pub id: String,
    pub ok: bool,
    pub latency_ms: u64,
    pub detail: String,
}

pub async fn run_entry(entry: &CheckEntry) -> CheckRun {
    run_ping(entry.id.as_str(), entry.host.as_str(), entry.timeout).await
}

async fn run_ping(id: &str, host: &str, timeout_ms: u64) -> CheckRun {
    let start = Instant::now();
    let ip = match resolve_host(host).await {
        Ok(ip) => ip,
        Err(e) => {
            return CheckRun {
                id: id.to_string(),
                ok: false,
                latency_ms: 0,
                detail: e,
            };
        }
    };

    let timeout = Duration::from_millis(timeout_ms.max(1));
    let payload = [0_u8; 8];
    match tokio::time::timeout(timeout, surge_ping::ping(ip, &payload)).await {
        Ok(Ok((_pkt, rtt))) => CheckRun {
            id: id.to_string(),
            ok: true,
            latency_ms: rtt.as_millis().min(u128::from(u64::MAX)) as u64,
            detail: format!("icmp echo from {host} ({ip})"),
        },
        Ok(Err(e)) => CheckRun {
            id: id.to_string(),
            ok: false,
            latency_ms: start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
            detail: e.to_string(),
        },
        Err(_) => CheckRun {
            id: id.to_string(),
            ok: false,
            latency_ms: timeout.as_millis().min(u128::from(u64::MAX)) as u64,
            detail: format!("icmp ping to {host} timed out after {timeout_ms} ms"),
        },
    }
}

async fn resolve_host(host: &str) -> Result<IpAddr, String> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ip);
    }

    let mut addrs = lookup_host((host, 0))
        .await
        .map_err(|e| format!("dns lookup for {host}: {e}"))?;

    addrs
        .next()
        .map(|a| a.ip())
        .ok_or_else(|| format!("no addresses found for {host}"))
}

pub async fn run_all_entries(entries: &[CheckEntry]) -> Vec<CheckRun> {
    let futures: Vec<_> = entries.iter().map(run_entry).collect();
    futures::future::join_all(futures).await
}
