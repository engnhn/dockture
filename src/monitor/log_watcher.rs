use crate::config::Config;
use crate::notifier::Notifier;
use bollard::Docker;
use bollard::container::{LogOutput, LogsOptions};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

pub type LogAlertCache = Arc<Mutex<HashMap<String, std::time::Instant>>>;

pub async fn monitor_container_logs(
    docker: Docker,
    container_id: String,
    container_name: String,
    config: Config,
    notifier: Notifier,
    cache: LogAlertCache,
) -> Result<(), String> {
    let keywords = match &config.log_keywords {
        Some(kw) => kw,
        None => &vec!["error".to_string(), "fatal".to_string(), "fail".to_string()],
    };
    if keywords.is_empty() {
        return Ok(());
    }

    let since = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let log_options = LogsOptions::<String> {
        follow: true,
        stdout: true,
        stderr: true,
        since,
        ..Default::default()
    };

    println!(
        "Log Monitor: Streaming logs for '{}' ({})",
        container_name, container_id
    );
    let mut logs_stream = docker.logs(&container_id, Some(log_options));

    while let Some(log_res) = logs_stream.next().await {
        match log_res {
            Ok(output) => {
                let text = match output {
                    LogOutput::StdOut { message } => String::from_utf8_lossy(&message).into_owned(),
                    LogOutput::StdErr { message } => String::from_utf8_lossy(&message).into_owned(),
                    LogOutput::Console { message } => {
                        String::from_utf8_lossy(&message).into_owned()
                    }
                    _ => String::new(),
                };

                for line in text.lines() {
                    let lower_line = line.to_lowercase();
                    for kw in keywords {
                        let lower_kw = kw.to_lowercase();
                        if lower_line.contains(&lower_kw) {
                            let cache_key = format!("{}:{}", container_name, kw);
                            {
                                let mut active_cache = cache.lock().await;
                                if let Some(last_sent) = active_cache.get(&cache_key) {
                                    if last_sent.elapsed() < std::time::Duration::from_secs(60) {
                                        continue;
                                    }
                                }
                                active_cache.insert(cache_key, std::time::Instant::now());
                            }

                            println!(
                                "Log Monitor: Container '{}' matched keyword '{}': {}",
                                container_name, kw, line
                            );

                            let alert_reason = format!("Log Keyword Match ('{}')", kw);
                            let subject = format!(
                                "[DOCKTURE LOG ALERT] Container '{}' -> {}",
                                container_name, alert_reason
                            );

                            let escaped_line = line
                                .replace('&', "&amp;")
                                .replace('<', "&lt;")
                                .replace('>', "&gt;")
                                .replace('"', "&quot;")
                                .replace('\'', "&#x27;");

                            let plain_body = format!(
                                "--- DOCKTURE LOG MATCH WARNING ---\n\
                                 Container Name: {}\n\
                                 Container ID: {}\n\
                                 Matched Keyword: {}\n\n\
                                 Matched Log Line:\n\
                                 {}\n",
                                container_name, container_id, kw, line
                            );

                            let meta = vec![
                                ("Container Name", container_name.clone()),
                                ("Container ID", container_id.clone()),
                                ("Matched Keyword", kw.clone()),
                                (
                                    "Log Trigger",
                                    "Matched keyword in container stdout/stderr".to_string(),
                                ),
                            ];

                            let html_body = crate::templates::render_html_report(
                                "Log Keyword Match Alert",
                                &alert_reason,
                                "#ef4444",
                                "#fef2f2",
                                &meta,
                                Some(&escaped_line),
                            );

                            if let Err(e) = notifier.send_notification(
                                "warning",
                                &subject,
                                &plain_body,
                                &html_body,
                            ) {
                                eprintln!("Failed to send log alert email: {}", e);
                            }

                            notifier
                                .send_webhook_alerts(
                                    "warning",
                                    &container_name,
                                    &alert_reason,
                                    "Log Trigger",
                                    line,
                                )
                                .await;
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }
    println!(
        "Log Monitor: Stopped streaming logs for '{}'",
        container_name
    );
    Ok(())
}

pub async fn harvest_container_logs(
    docker: &Docker,
    container_id: &str,
    tail_size: usize,
) -> String {
    let log_options = LogsOptions::<String> {
        stdout: true,
        stderr: true,
        tail: tail_size.to_string(),
        ..Default::default()
    };

    let mut logs_stream = docker.logs(container_id, Some(log_options));
    let mut log_lines = Vec::new();

    while let Some(log_res) = logs_stream.next().await {
        match log_res {
            Ok(output) => {
                let text = match output {
                    LogOutput::StdOut { message } => String::from_utf8_lossy(&message).into_owned(),
                    LogOutput::StdErr { message } => String::from_utf8_lossy(&message).into_owned(),
                    LogOutput::Console { message } => {
                        String::from_utf8_lossy(&message).into_owned()
                    }
                    _ => String::new(),
                };
                log_lines.push(text);
            }
            Err(e) => {
                log_lines.push(format!("\n[Error fetching logs: {}]\n", e));
                break;
            }
        }
    }

    if log_lines.is_empty() {
        "[No logs available]".to_string()
    } else {
        log_lines.join("")
    }
}
