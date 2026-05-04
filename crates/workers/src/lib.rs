//! Background check runner: runs configured probes on an interval and pushes status to Uptime Kuma.

mod checks;
mod push;

use std::sync::Arc;
use std::time::Duration;

use app_config::AppConfig;
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};

pub use checks::{CheckRun, run_all_checks};

/// Runs one full cycle: all checks in parallel, then a single push with aggregate result.
pub async fn run_once(config: &AppConfig) -> anyhow::Result<()> {
    let runs = run_all_checks(&config.checks).await;
    if !runs.iter().all(|r| r.ok) {
        for run in &runs {
            if !run.ok {
                warn!(id = %run.id, detail = %run.detail, "check failed");
            }
        }
    } else {
        debug!(checks = config.checks.len(), "all checks passed");
    }

    push::send_push(config, &runs).await
}

/// Spawns a task that runs [`run_once`] every `config.agent.interval` seconds until aborted.
pub fn spawn(config: Arc<AppConfig>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let period = Duration::from_secs(config.agent.interval.max(1));
        let mut ticker = tokio::time::interval(period);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            ticker.tick().await;
            match run_once(&config).await {
                Ok(()) => debug!("check cycle finished"),
                Err(e) => error!(error = %e, "check cycle finished with errors"),
            }
        }
    })
}
