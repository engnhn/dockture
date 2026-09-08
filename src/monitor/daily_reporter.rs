use crate::config::Config;
use crate::notifier::Notifier;
use bollard::Docker;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct DailyStats {
    pub date: String,
    pub crashes: u32,
    pub health_failures: u32,
    pub log_matches: u32,
    pub resource_warnings: u32,
    pub anomalies: u32,
    pub container_alerts: HashMap<String, u32>,
    pub last_report_epoch: u64,
}

pub type SharedDailyStats = Arc<Mutex<DailyStats>>;

pub fn new_shared_daily_stats() -> SharedDailyStats {
    Arc::new(Mutex::new(DailyStats::default()))
}

pub async fn record_event(stats: &SharedDailyStats, event_type: &str, container_name: Option<&str>) {
    let mut guard = stats.lock().await;
    match event_type {
        "crash" => guard.crashes += 1,
        "health" => guard.health_failures += 1,
        "log_match" => guard.log_matches += 1,
        "resource_warning" => guard.resource_warnings += 1,
        "anomaly" => guard.anomalies += 1,
        _ => {}
    }
    if let Some(c_name) = container_name {
        *guard.container_alerts.entry(c_name.to_string()).or_insert(0) += 1;
    }
}

pub async fn generate_and_send_daily_report(
    docker: &Docker,
    config: &Config,
    notifier: &Notifier,
    stats: &SharedDailyStats,
) -> Result<(), String> {
    let list_options = Some(bollard::container::ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    });

    let containers = docker
        .list_containers(list_options)
        .await
        .map_err(|e| format!("Failed to list containers for daily report: {}", e))?;

    let mut container_rows = Vec::new();
    let mut total_monitored = 0;

    let snapshot = {
        let guard = stats.lock().await;
        guard.clone()
    };

    for c in &containers {
        let name = c
            .names
            .as_ref()
            .and_then(|names| names.first())
            .map(|n| n.trim_start_matches('/'))
            .unwrap_or("unknown");

        if name == "unknown" || !config.is_container_monitored(name) {
            continue;
        }

        total_monitored += 1;
        let state = c.state.as_deref().unwrap_or("unknown");
        let alert_count = snapshot.container_alerts.get(name).copied().unwrap_or(0);
        container_rows.push((name, state, alert_count));
    }

    let disk_path = if std::path::Path::new("/var/lib/docker").exists() {
        "/var/lib/docker"
    } else {
        "/"
    };

    let disk_usage_info = match crate::utils::get_disk_usage(disk_path).await {
        Ok((used, total)) if total > 0 => {
            let pct = (used as f64 / total as f64) * 100.0;
            let used_gb = used as f64 / (1024.0 * 1024.0 * 1024.0);
            let total_gb = total as f64 / (1024.0 * 1024.0 * 1024.0);
            format!("{:.2} GB / {:.2} GB ({:.1}%)", used_gb, total_gb, pct)
        }
        _ => "Unavailable".to_string(),
    };

    let now_date = chrono::Local::now().format("%Y-%m-%d").to_string();

    let container_row_refs: Vec<(&str, &str, u32)> = container_rows
        .iter()
        .map(|(n, s, c)| (*n, *s, *c))
        .collect();

    let html_body = crate::templates::render_daily_report_html(
        &now_date,
        total_monitored,
        snapshot.crashes,
        snapshot.health_failures,
        snapshot.log_matches,
        snapshot.resource_warnings,
        snapshot.anomalies,
        &disk_usage_info,
        &container_row_refs,
    );

    let plain_body = format!(
        "--- DOCKTURE DAILY HEALTH SUMMARY REPORT ---\n\
         Date: {}\n\
         Monitored Containers: {}\n\
         Crashes / OOM Kills: {}\n\
         Health Failures: {}\n\
         Log Error Matches: {}\n\
         Resource Warnings: {}\n\
         Anomalies Detected: {}\n\
         Host Storage Usage: {}\n",
        now_date,
        total_monitored,
        snapshot.crashes,
        snapshot.health_failures,
        snapshot.log_matches,
        snapshot.resource_warnings,
        snapshot.anomalies,
        disk_usage_info
    );

    let subject = format!("[DOCKTURE DAILY REPORT] System Health Summary ({})", now_date);

    notifier.send_notification("daily_report", &subject, &plain_body, &html_body)?;

    {
        let mut guard = stats.lock().await;
        guard.crashes = 0;
        guard.health_failures = 0;
        guard.log_matches = 0;
        guard.resource_warnings = 0;
        guard.anomalies = 0;
        guard.container_alerts.clear();
        guard.date = now_date;
        guard.last_report_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    Ok(())
}

pub async fn run_daily_reporter(
    docker: Docker,
    config: Config,
    notifier: Notifier,
    stats: SharedDailyStats,
) {
    if !config.daily_report_enabled() {
        return;
    }

    loop {
        sleep(std::time::Duration::from_secs(60)).await;

        if !config.daily_report_enabled() {
            continue;
        }

        let now_local = chrono::Local::now();
        let target_time_str = config.daily_report_time();
        let current_time_str = now_local.format("%H:%M").to_string();

        let now_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let (should_trigger, last_epoch) = {
            let guard = stats.lock().await;
            let time_matches = current_time_str == target_time_str;
            let day_elapsed = now_epoch.saturating_sub(guard.last_report_epoch) >= 86000;
            let min_cooldown = now_epoch.saturating_sub(guard.last_report_epoch) >= 3600;
            ( (time_matches && min_cooldown) || (day_elapsed && guard.last_report_epoch > 0), guard.last_report_epoch )
        };

        if last_epoch == 0 {
            let mut guard = stats.lock().await;
            guard.last_report_epoch = now_epoch;
            continue;
        }

        if should_trigger {
            println!("Daily Reporter: Generating automated 24-hour summary report...");
            if let Err(e) = generate_and_send_daily_report(&docker, &config, &notifier, &stats).await {
                eprintln!("Daily Reporter: Error dispatching daily report: {}", e);
            }
        }
    }
}
