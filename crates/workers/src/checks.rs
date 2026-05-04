use std::net::IpAddr;
use std::time::{Duration, Instant};

use app_config::{CheckEntry, CheckProbe};
use reqwest::header;
use reqwest::{Method, redirect};
use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub struct CheckRun {
    pub id: String,
    pub ok: bool,
    pub latency_ms: u64,
    pub detail: String,
}

pub async fn run_entry(entry: &CheckEntry) -> CheckRun {
    let id = entry.id.as_str();
    match &entry.probe {
        CheckProbe::Http {
            url,
            method,
            expected_status,
            timeout,
            follow_redirects,
        } => {
            run_http_like(
                id,
                url,
                method,
                expected_status,
                *timeout,
                *follow_redirects,
                false,
            )
            .await
        }
        CheckProbe::Https {
            url,
            method,
            expected_status,
            timeout,
            insecure_skip_verify,
        } => {
            run_http_like(
                id,
                url,
                method,
                expected_status,
                *timeout,
                true,
                *insecure_skip_verify,
            )
            .await
        }
        CheckProbe::Tcp {
            host,
            port,
            timeout,
        } => run_tcp(id, host, *port, *timeout).await,
        CheckProbe::Ping { host, timeout } => run_ping(id, host, *timeout).await,
    }
}

async fn run_http_like(
    id: &str,
    url: &str,
    method: &str,
    expected_status: &[u16],
    timeout_ms: u64,
    follow_redirects: bool,
    insecure_skip_verify: bool,
) -> CheckRun {
    let start = Instant::now();
    let method = match Method::from_bytes(method.trim().as_bytes()) {
        Ok(m) => m,
        Err(_) => {
            return CheckRun {
                id: id.to_string(),
                ok: false,
                latency_ms: 0,
                detail: format!("unsupported HTTP method: {method}"),
            };
        }
    };

    let redirect = if follow_redirects {
        redirect::Policy::limited(10)
    } else {
        redirect::Policy::none()
    };

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms.max(1)))
        .redirect(redirect)
        .danger_accept_invalid_certs(insecure_skip_verify)
        .user_agent("ukuma-push-agent")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return CheckRun {
                id: id.to_string(),
                ok: false,
                latency_ms: 0,
                detail: format!("http client build failed: {e}"),
            };
        }
    };

    let req = client
        .request(method, url)
        .header(header::ACCEPT, "*/*");

    match req.send().await {
        Ok(resp) => {
            let code = resp.status().as_u16();
            let ok = expected_status.contains(&code);
            let latency_ms = start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            let detail = if ok {
                format!("status {code}")
            } else {
                format!("status {code}, expected one of {expected_status:?}")
            };
            CheckRun {
                id: id.to_string(),
                ok,
                latency_ms,
                detail,
            }
        }
        Err(e) => CheckRun {
            id: id.to_string(),
            ok: false,
            latency_ms: start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
            detail: e.to_string(),
        },
    }
}

async fn run_tcp(id: &str, host: &str, port: u16, timeout_ms: u64) -> CheckRun {
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms.max(1));
    match tokio::time::timeout(timeout, TcpStream::connect((host, port))).await {
        Ok(Ok(_)) => CheckRun {
            id: id.to_string(),
            ok: true,
            latency_ms: start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
            detail: format!("connected to {host}:{port}"),
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
            detail: format!("connect to {host}:{port} timed out after {timeout_ms} ms"),
        },
    }
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

    let mut addrs = tokio::net::lookup_host((host, 0))
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
