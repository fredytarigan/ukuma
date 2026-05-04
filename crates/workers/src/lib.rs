//! Background runner: ICMP ping targets on an interval, then pushes each result to its own Uptime Kuma Push monitor.

mod checks;
mod push;

use std::sync::Arc;
use std::time::Duration;

use app_config::AppConfig;
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};

pub use checks::{CheckRun, run_all_entries, run_entry};

/// Runs one full cycle: for every check, run the probe then push to that monitor’s push URL (in parallel across checks).
pub async fn run_once(config: &AppConfig) -> anyhow::Result<()> {
    let push = config.push.clone();
    let futures: Vec<_> = config
        .checks
        .iter()
        .cloned()
        .map(move |entry| {
            let push = push.clone();
            async move {
                let run = checks::run_entry(&entry).await;
                if !run.ok {
                    warn!(id = %run.id, detail = %run.detail, "check failed");
                }
                let push_res =
                    push::send_push_for_check(&push, entry.push_token.as_str(), &run).await;
                (run, push_res)
            }
        })
        .collect();

    let outcomes = futures::future::join_all(futures).await;
    let mut failed_pushes = Vec::new();

    for (run, res) in outcomes {
        if let Err(e) = res {
            error!(id = %run.id, error = %e, "push failed");
            failed_pushes.push(format!("{}: {e}", run.id));
        }
    }

    if failed_pushes.is_empty() {
        debug!(checks = config.checks.len(), "check cycle finished");
        Ok(())
    } else {
        Err(anyhow::anyhow!(failed_pushes.join("; ")))
    }
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
