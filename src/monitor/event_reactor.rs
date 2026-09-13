use crate::config::Config;
use crate::notifier::Notifier;
use bollard::Docker;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::watch;

pub struct EventContext<'a> {
    pub docker: &'a Docker,
    pub config: &'a Config,
    pub notifier: &'a Notifier,
    pub log_alert_cache: &'a super::log_watcher::LogAlertCache,
    pub restart_tracker: &'a super::self_healer::RestartTracker,
    pub daily_stats: &'a super::daily_reporter::SharedDailyStats,
    pub shutdown: &'a watch::Receiver<bool>,
    pub task_handles: &'a super::TaskHandles,
    pub active_log_watchers: &'a super::ActiveLogWatchers,
}

pub async fn handle_docker_event(
    ctx: EventContext<'_>,
    event: bollard::models::EventMessage,
) -> Result<(), String> {
    let EventContext {
        docker,
        config,
        notifier,
        log_alert_cache,
        restart_tracker,
        daily_stats,
        shutdown,
        task_handles,
        active_log_watchers,
    } = ctx;

    let action = event.action.as_deref().unwrap_or("");
    let actor = match &event.actor {
        Some(a) => a,
        None => return Ok(()),
    };

    let container_id = actor.id.as_deref().unwrap_or("");
    if container_id.is_empty() {
        return Ok(());
    }

    let container_name = actor
        .attributes
        .as_ref()
        .and_then(|attrs| attrs.get("name").map(|s| s.as_str()))
        .unwrap_or("unknown");

    if !config.is_container_monitored(container_name) {
        return Ok(());
    }

    if action == "start" {
        println!(
            "Log Monitor: Newly started container '{}' detected. Spawning log monitor...",
            container_name
        );
        let spec = super::log_watcher::LogWatcherSpec {
            docker: docker.clone(),
            container_id: container_id.to_string(),
            container_name: container_name.to_string(),
            config: config.clone(),
            notifier: notifier.clone(),
            cache: log_alert_cache.clone(),
            daily_stats: daily_stats.clone(),
            shutdown: shutdown.clone(),
        };
        super::spawn_log_watcher(spec, task_handles, active_log_watchers).await;
        return Ok(());
    }

    let timestamp = event.time.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    });

    let mut should_alert = false;
    let mut alert_reason = String::new();
    let mut alert_type = "crash";

    if action == "oom" {
        should_alert = true;
        alert_reason = "Out of Memory (OOM) Killed".to_string();
        alert_type = "crash";
    } else if action == "die" {
        let exit_code = actor
            .attributes
            .as_ref()
            .and_then(|attrs| attrs.get("exitCode").map(|s| s.as_str()))
            .unwrap_or("unknown");

        if exit_code != "0" && exit_code != "143" {
            should_alert = true;
            alert_reason = format!("Crashed (Exit Code: {})", exit_code);
            alert_type = "crash";
        }
    } else if action.starts_with("health_status: unhealthy") {
        should_alert = true;
        alert_reason = "Health Status became UNHEALTHY".to_string();
        alert_type = "health";
    }

    if !should_alert {
        return Ok(());
    }

    super::daily_reporter::record_event(daily_stats, alert_type, Some(container_name)).await;

    let self_healing_res = super::self_healer::handle_self_healing(
        docker,
        container_id,
        container_name,
        &alert_reason,
        config.auto_restart.unwrap_or(false),
        restart_tracker,
    )
    .await;

    alert_reason = self_healing_res.updated_reason;
    let self_healing_status = self_healing_res.status;
    let is_crash_loop = self_healing_res.is_crash_loop;

    println!(
        "ALERT TRIGGERED: Container '{}' ({}) -> {} [Self-healing: {}]",
        container_name, container_id, alert_reason, self_healing_status
    );

    let logs =
        super::log_watcher::harvest_container_logs(docker, container_id, config.log_tail_size)
            .await;

    let escaped_logs = logs
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;");

    let subject_prefix = if is_crash_loop {
        "[DOCKTURE CRITICAL]"
    } else {
        "[DOCKTURE ALERT]"
    };
    let subject = format!(
        "{} Container '{}' -> {}",
        subject_prefix, container_name, alert_reason
    );

    let plain_body = format!(
        "--- DOCKTURE ALERT REPORT ---\n\
         Container Name: {}\n\
         Container ID: {}\n\
         Event/Reason: {}\n\
         Timestamp: {}\n\
         Self-Healing Action: {}\n\n\
         --- DIAGNOSTIC LOGS (Last {} lines) ---\n\
         {}\n",
        container_name,
        container_id,
        alert_reason,
        timestamp,
        self_healing_status,
        config.log_tail_size,
        logs
    );

    let meta = vec![
        ("Container Name", container_name.to_string()),
        ("Container ID", container_id.to_string()),
        ("Event / Reason", alert_reason.clone()),
        ("Timestamp (Epoch)", timestamp.to_string()),
        ("Self-Healing Status", self_healing_status),
    ];

    let (theme_color, theme_bg) = if is_crash_loop {
        ("#7f1d1d", "#fef2f2")
    } else if alert_type == "crash" {
        ("#ef4444", "#fef2f2")
    } else if alert_type == "warning" {
        ("#d97706", "#fffbeb")
    } else {
        ("#e11d48", "#fff1f2")
    };

    let html_report_title = if is_crash_loop {
        "CRITICAL CrashLoopBackOff Alert"
    } else {
        "Container Diagnostic Alert"
    };

    let html_body = crate::templates::render_html_report(
        html_report_title,
        &alert_reason,
        theme_color,
        theme_bg,
        &meta,
        Some(&escaped_logs),
    );

    if let Err(e) = notifier.send_notification(alert_type, &subject, &plain_body, &html_body) {
        eprintln!("Failed to send notification email: {}", e);
    }

    notifier
        .send_webhook_alerts(
            alert_type,
            container_name,
            &alert_reason,
            &timestamp.to_string(),
            &logs,
        )
        .await;

    Ok(())
}
