use crate::config::Config;
use crate::notifier::Notifier;
use bollard::Docker;
use bollard::container::{ListContainersOptions, StatsOptions};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone, Default)]
struct MetricHistory {
    cpu: Vec<f64>,
    memory: Vec<f64>,
}

pub fn calculate_z_score(history: &[f64], current: f64, min_std_dev: f64) -> Option<f64> {
    if history.len() < 5 {
        return None;
    }

    let sum: f64 = history.iter().sum();
    let mean = sum / history.len() as f64;

    let variance: f64 = history
        .iter()
        .map(|&x| {
            let diff = x - mean;
            diff * diff
        })
        .sum::<f64>()
        / history.len() as f64;

    let std_dev = variance.sqrt();
    let std_dev = if std_dev < min_std_dev {
        min_std_dev
    } else {
        std_dev
    };

    Some((current - mean) / std_dev)
}

async fn trigger_anomaly_alert(
    container_name: &str,
    container_id: &str,
    metric_name: &str,
    current_value: f64,
    historical_mean: f64,
    z_score: f64,
    notifier: &Notifier,
) {
    let alert_reason = format!(
        "Resource Anomaly ({}): {:.1}% (Avg: {:.1}%)",
        metric_name, current_value, historical_mean
    );
    let subject = format!(
        "[DOCKTURE ANOMALY] Container '{}' -> {}",
        container_name, alert_reason
    );

    let plain_body = format!(
        "--- DOCKTURE RESOURCE ANOMALY ALERT ---\n\
         Container Name: {}\n\
         Container ID: {}\n\
         Metric: {}\n\
         Current Value: {:.1}%\n\
         Historical Average: {:.1}%\n\
         Z-Score Deviation: {:.2} std dev\n\n\
         Status: Significant deviation from normal pattern detected.",
        container_name, container_id, metric_name, current_value, historical_mean, z_score
    );

    let meta = vec![
        ("Container Name", container_name.to_string()),
        ("Container ID", container_id.to_string()),
        ("Metric", metric_name.to_string()),
        ("Current Value", format!("{:.1}%", current_value)),
        ("Historical Average", format!("{:.1}%", historical_mean)),
        ("Z-Score Deviation", format!("{:.2} std dev", z_score)),
    ];

    let html_body = crate::templates::render_html_report(
        "Resource Anomaly Alert",
        &alert_reason,
        "#d97706",
        "#fffbeb",
        &meta,
        None,
    );

    if let Err(e) = notifier.send_notification("warning", &subject, &plain_body, &html_body) {
        eprintln!("Failed to send anomaly warning email: {}", e);
    }

    notifier
        .send_webhook_alerts(
            "warning",
            container_name,
            &alert_reason,
            "Resource Anomaly",
            "",
        )
        .await;
}

pub async fn run_resource_monitor(
    docker: Docker,
    config: Config,
    notifier: Notifier,
) -> Result<(), String> {
    let mut mem_warning_states: HashMap<String, bool> = HashMap::new();
    let mut cpu_warning_states: HashMap<String, bool> = HashMap::new();
    let mut disk_warning_sent = false;
    let mut metric_history: HashMap<String, MetricHistory> = HashMap::new();
    let mut last_anomaly_alerts: HashMap<String, std::time::Instant> = HashMap::new();

    loop {
        sleep(Duration::from_secs(30)).await;

        let list_options = Some(ListContainersOptions::<String> {
            all: false,
            ..Default::default()
        });

        let containers = match docker.list_containers(list_options).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Resource monitor: failed to list containers: {}", e);
                continue;
            }
        };

        for container in containers {
            let container_id = match &container.id {
                Some(id) => id,
                None => continue,
            };

            let container_name = container
                .names
                .as_ref()
                .and_then(|names| names.first().map(|n| n.trim_start_matches('/')))
                .unwrap_or("unknown");

            if container_name == "unknown" {
                continue;
            }

            if !config.is_container_monitored(container_name) {
                continue;
            }

            let mut stats_stream = docker.stats(
                container_id,
                Some(StatsOptions {
                    stream: false,
                    one_shot: true,
                }),
            );
            if let Some(stats_res) = stats_stream.next().await {
                let stats = match stats_res {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                if let (Some(usage), Some(limit)) =
                    (stats.memory_stats.usage, stats.memory_stats.limit)
                {
                    if limit > 0 {
                        let pct = (usage as f64 / limit as f64) * 100.0;
                        let usage_mb = usage as f64 / (1024.0 * 1024.0);
                        let limit_mb = limit as f64 / (1024.0 * 1024.0);

                        let warning_sent = mem_warning_states
                            .entry(container_name.to_string())
                            .or_insert(false);

                        if pct >= 90.0 {
                            if !*warning_sent {
                                *warning_sent = true;
                                println!(
                                    "RESOURCE WARNING (Memory): Container '{}' memory usage is at {:.1}% ({:.1} MB / {:.1} MB)",
                                    container_name, pct, usage_mb, limit_mb
                                );

                                let alert_reason =
                                    format!("High Memory Usage Warning ({:.1}%)", pct);
                                let subject = format!(
                                    "[DOCKTURE WARNING] Container '{}' -> {}",
                                    container_name, alert_reason
                                );

                                let plain_body = format!(
                                    "--- DOCKTURE RESOURCE WARNING ---\n\
                                     Container Name: {}\n\
                                     Container ID: {}\n\
                                     Memory Usage: {:.2} MB / {:.2} MB ({:.2}%)\n\n\
                                     Status: memory usage exceeds 90% threshold. Please inspect container behavior.",
                                    container_name, container_id, usage_mb, limit_mb, pct
                                );

                                let meta = vec![
                                    ("Container Name", container_name.to_string()),
                                    ("Container ID", container_id.to_string()),
                                    (
                                        "Memory Usage",
                                        format!("{:.2} MB / {:.2} MB", usage_mb, limit_mb),
                                    ),
                                    ("Usage Percentage", format!("{:.1}%", pct)),
                                    ("Status", "HIGH MEMORY WARNING (exceeds 90%)".to_string()),
                                ];

                                let html_body = crate::templates::render_html_report(
                                    "Resource Utilization Alert (Memory)",
                                    &alert_reason,
                                    "#d97706",
                                    "#fffbeb",
                                    &meta,
                                    None,
                                );

                                if let Err(e) = notifier.send_notification(
                                    "warning",
                                    &subject,
                                    &plain_body,
                                    &html_body,
                                ) {
                                    eprintln!("Failed to send memory warning email: {}", e);
                                }

                                notifier
                                    .send_webhook_alerts(
                                        "warning",
                                        container_name,
                                        &alert_reason,
                                        "Resource Warning",
                                        "",
                                    )
                                    .await;
                            }
                        } else if pct < 80.0 {
                            if *warning_sent {
                                *warning_sent = false;
                                println!(
                                    "RESOURCE RECOVERY (Memory): Container '{}' memory usage recovered to {:.1}% ({:.1} MB / {:.1} MB)",
                                    container_name, pct, usage_mb, limit_mb
                                );

                                let alert_reason = format!("Memory Usage Recovered ({:.1}%)", pct);
                                let subject = format!(
                                    "[DOCKTURE RECOVERY] Container '{}' -> {}",
                                    container_name, alert_reason
                                );

                                let plain_body = format!(
                                    "--- DOCKTURE RESOURCE RECOVERY ---\n\
                                     Container Name: {}\n\
                                     Container ID: {}\n\
                                     Memory Usage: {:.2} MB / {:.2} MB ({:.2}%)\n\n\
                                     Status: memory usage returned to normal levels (below 80%).",
                                    container_name, container_id, usage_mb, limit_mb, pct
                                );

                                let meta = vec![
                                    ("Container Name", container_name.to_string()),
                                    ("Container ID", container_id.to_string()),
                                    (
                                        "Memory Usage",
                                        format!("{:.2} MB / {:.2} MB", usage_mb, limit_mb),
                                    ),
                                    ("Usage Percentage", format!("{:.1}%", pct)),
                                    ("Status", "MEMORY USAGE RECOVERED (below 80%)".to_string()),
                                ];

                                let html_body = crate::templates::render_html_report(
                                    "Resource Recovery Notification (Memory)",
                                    &alert_reason,
                                    "#10b981",
                                    "#f0fdf4",
                                    &meta,
                                    None,
                                );

                                if let Err(e) = notifier.send_notification(
                                    "recovery",
                                    &subject,
                                    &plain_body,
                                    &html_body,
                                ) {
                                    eprintln!("Failed to send memory recovery email: {}", e);
                                }

                                notifier
                                    .send_webhook_alerts(
                                        "recovery",
                                        container_name,
                                        &alert_reason,
                                        "Resource Recovery",
                                        "",
                                    )
                                    .await;
                            }
                        }

                        if config.anomaly_detection() {
                            let (mean, z_score) = {
                                let history = metric_history
                                    .entry(container_name.to_string())
                                    .or_default();
                                let z = calculate_z_score(
                                    &history.memory,
                                    pct,
                                    config.anomaly_sensitivity(),
                                );
                                let mean = if history.memory.is_empty() {
                                    0.0
                                } else {
                                    history.memory.iter().sum::<f64>() / history.memory.len() as f64
                                };
                                (mean, z)
                            };

                            if let Some(z) = z_score {
                                if z > config.anomaly_threshold() && pct > mean {
                                    let now = std::time::Instant::now();
                                    let cooldown_key = format!("{}-Memory", container_name);
                                    let is_cooldown = last_anomaly_alerts
                                        .get(&cooldown_key)
                                        .map(|last_alert| {
                                            now.duration_since(*last_alert)
                                                < std::time::Duration::from_secs(300)
                                        })
                                        .unwrap_or(false);

                                    if !is_cooldown {
                                        last_anomaly_alerts.insert(cooldown_key, now);
                                        trigger_anomaly_alert(
                                            container_name,
                                            container_id,
                                            "Memory",
                                            pct,
                                            mean,
                                            z,
                                            &notifier,
                                        )
                                        .await;
                                    }
                                }
                            }

                            let history = metric_history
                                .entry(container_name.to_string())
                                .or_default();
                            history.memory.push(pct);
                            if history.memory.len() > 60 {
                                history.memory.remove(0);
                            }
                        }
                    }
                }

                let cpu_usage = stats.cpu_stats.cpu_usage.total_usage;
                let precpu_usage = stats.precpu_stats.cpu_usage.total_usage;
                let system_cpu = stats.cpu_stats.system_cpu_usage;
                let presystem_cpu = stats.precpu_stats.system_cpu_usage;

                if let (Some(sys_cpu), Some(pre_sys_cpu)) = (system_cpu, presystem_cpu) {
                    let cpu_delta = cpu_usage.saturating_sub(precpu_usage) as f64;
                    let system_delta = sys_cpu.saturating_sub(pre_sys_cpu) as f64;
                    let online_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;

                    if system_delta > 0.0 && cpu_delta >= 0.0 {
                        let cpu_pct = (cpu_delta / system_delta) * online_cpus * 100.0;

                        let cpu_warning_sent = cpu_warning_states
                            .entry(container_name.to_string())
                            .or_insert(false);

                        if cpu_pct >= 90.0 {
                            if !*cpu_warning_sent {
                                *cpu_warning_sent = true;
                                println!(
                                    "RESOURCE WARNING (CPU): Container '{}' CPU usage is at {:.1}%",
                                    container_name, cpu_pct
                                );

                                let alert_reason =
                                    format!("High CPU Usage Warning ({:.1}%)", cpu_pct);
                                let subject = format!(
                                    "[DOCKTURE WARNING] Container '{}' -> {}",
                                    container_name, alert_reason
                                );

                                let plain_body = format!(
                                    "--- DOCKTURE RESOURCE WARNING ---\n\
                                     Container Name: {}\n\
                                     Container ID: {}\n\
                                     CPU Usage: {:.2}%\n\n\
                                     Status: CPU usage exceeds 90% threshold. Please inspect container activity.",
                                    container_name, container_id, cpu_pct
                                );

                                let meta = vec![
                                    ("Container Name", container_name.to_string()),
                                    ("Container ID", container_id.to_string()),
                                    ("CPU Usage", format!("{:.1}%", cpu_pct)),
                                    ("Status", "HIGH CPU WARNING (exceeds 90%)".to_string()),
                                ];

                                let html_body = crate::templates::render_html_report(
                                    "Resource Utilization Alert (CPU)",
                                    &alert_reason,
                                    "#d97706",
                                    "#fffbeb",
                                    &meta,
                                    None,
                                );

                                if let Err(e) = notifier.send_notification(
                                    "warning",
                                    &subject,
                                    &plain_body,
                                    &html_body,
                                ) {
                                    eprintln!("Failed to send CPU warning email: {}", e);
                                }

                                notifier
                                    .send_webhook_alerts(
                                        "warning",
                                        container_name,
                                        &alert_reason,
                                        "Resource Warning",
                                        "",
                                    )
                                    .await;
                            }
                        } else if cpu_pct < 80.0 {
                            if *cpu_warning_sent {
                                *cpu_warning_sent = false;
                                println!(
                                    "RESOURCE RECOVERY (CPU): Container '{}' CPU usage recovered to {:.1}%",
                                    container_name, cpu_pct
                                );

                                let alert_reason = format!("CPU Usage Recovered ({:.1}%)", cpu_pct);
                                let subject = format!(
                                    "[DOCKTURE RECOVERY] Container '{}' -> {}",
                                    container_name, alert_reason
                                );

                                let plain_body = format!(
                                    "--- DOCKTURE RESOURCE RECOVERY ---\n\
                                     Container Name: {}\n\
                                     Container ID: {}\n\
                                     CPU Usage: {:.2}%\n\n\
                                     Status: CPU usage returned to normal levels (below 80%).",
                                    container_name, container_id, cpu_pct
                                );

                                let meta = vec![
                                    ("Container Name", container_name.to_string()),
                                    ("Container ID", container_id.to_string()),
                                    ("CPU Usage", format!("{:.1}%", cpu_pct)),
                                    ("Status", "CPU USAGE RECOVERED (below 80%)".to_string()),
                                ];

                                let html_body = crate::templates::render_html_report(
                                    "Resource Recovery Notification (CPU)",
                                    &alert_reason,
                                    "#10b981",
                                    "#f0fdf4",
                                    &meta,
                                    None,
                                );

                                if let Err(e) = notifier.send_notification(
                                    "recovery",
                                    &subject,
                                    &plain_body,
                                    &html_body,
                                ) {
                                    eprintln!("Failed to send CPU recovery email: {}", e);
                                }

                                notifier
                                    .send_webhook_alerts(
                                        "recovery",
                                        container_name,
                                        &alert_reason,
                                        "Resource Recovery",
                                        "",
                                    )
                                    .await;
                            }
                        }

                        if config.anomaly_detection() {
                            let (mean, z_score) = {
                                let history = metric_history
                                    .entry(container_name.to_string())
                                    .or_default();
                                let z = calculate_z_score(
                                    &history.cpu,
                                    cpu_pct,
                                    config.anomaly_sensitivity(),
                                );
                                let mean = if history.cpu.is_empty() {
                                    0.0
                                } else {
                                    history.cpu.iter().sum::<f64>() / history.cpu.len() as f64
                                };
                                (mean, z)
                            };

                            if let Some(z) = z_score {
                                if z > config.anomaly_threshold() && cpu_pct > mean {
                                    let now = std::time::Instant::now();
                                    let cooldown_key = format!("{}-CPU", container_name);
                                    let is_cooldown = last_anomaly_alerts
                                        .get(&cooldown_key)
                                        .map(|last_alert| {
                                            now.duration_since(*last_alert)
                                                < std::time::Duration::from_secs(300)
                                        })
                                        .unwrap_or(false);

                                    if !is_cooldown {
                                        last_anomaly_alerts.insert(cooldown_key, now);
                                        trigger_anomaly_alert(
                                            container_name,
                                            container_id,
                                            "CPU",
                                            cpu_pct,
                                            mean,
                                            z,
                                            &notifier,
                                        )
                                        .await;
                                    }
                                }
                            }

                            let history = metric_history
                                .entry(container_name.to_string())
                                .or_default();
                            history.cpu.push(cpu_pct);
                            if history.cpu.len() > 60 {
                                history.cpu.remove(0);
                            }
                        }
                    }
                }
            }
        }

        let disk_path = if std::path::Path::new("/var/lib/docker").exists() {
            "/var/lib/docker"
        } else {
            "/"
        };

        if let Ok((used, total)) = crate::utils::get_disk_usage(disk_path).await {
            if total > 0 {
                let pct = (used as f64 / total as f64) * 100.0;
                let used_gb = used as f64 / (1024.0 * 1024.0 * 1024.0);
                let total_gb = total as f64 / (1024.0 * 1024.0 * 1024.0);

                if pct >= 90.0 {
                    if !disk_warning_sent {
                        disk_warning_sent = true;
                        println!(
                            "RESOURCE WARNING (Host Disk): Host disk space ({}) is at {:.1}% ({:.2} GB / {:.2} GB)",
                            disk_path, pct, used_gb, total_gb
                        );

                        let alert_reason =
                            format!("High Host Disk Space Usage Warning ({:.1}%)", pct);
                        let subject = format!("[DOCKTURE WARNING] Host System -> {}", alert_reason);

                        let plain_body = format!(
                            "--- DOCKTURE HOST DISK WARNING ---\n\
                             Path Monitored: {}\n\
                             Disk Space Usage: {:.2} GB / {:.2} GB ({:.1}%)\n\n\
                             Status: host disk usage exceeds 90% threshold. Please clean up unused images/volumes.",
                            disk_path, used_gb, total_gb, pct
                        );

                        let meta = vec![
                            ("Monitored Path", disk_path.to_string()),
                            (
                                "Disk Usage",
                                format!("{:.2} GB / {:.2} GB", used_gb, total_gb),
                            ),
                            ("Usage Percentage", format!("{:.1}%", pct)),
                            ("Status", "HIGH HOST DISK WARNING (exceeds 90%)".to_string()),
                        ];

                        let html_body = crate::templates::render_html_report(
                            "Host Storage Utilization Alert",
                            &alert_reason,
                            "#d97706",
                            "#fffbeb",
                            &meta,
                            None,
                        );

                        if let Err(e) =
                            notifier.send_notification("warning", &subject, &plain_body, &html_body)
                        {
                            eprintln!("Failed to send disk space warning email: {}", e);
                        }

                        notifier
                            .send_webhook_alerts(
                                "warning",
                                "Host Server",
                                &alert_reason,
                                "Host Disk Trigger",
                                "",
                            )
                            .await;
                    }
                } else if pct < 80.0 {
                    if disk_warning_sent {
                        disk_warning_sent = false;
                        println!(
                            "RESOURCE RECOVERY (Host Disk): Host disk space ({}) recovered to {:.1}% ({:.2} GB / {:.2} GB)",
                            disk_path, pct, used_gb, total_gb
                        );

                        let alert_reason = format!("Host Disk Space Usage Recovered ({:.1}%)", pct);
                        let subject =
                            format!("[DOCKTURE RECOVERY] Host System -> {}", alert_reason);

                        let plain_body = format!(
                            "--- DOCKTURE HOST DISK RECOVERY ---\n\
                             Path Monitored: {}\n\
                             Disk Space Usage: {:.2} GB / {:.2} GB ({:.1}%)\n\n\
                             Status: host disk usage returned to normal levels (below 80%).",
                            disk_path, used_gb, total_gb, pct
                        );

                        let meta = vec![
                            ("Monitored Path", disk_path.to_string()),
                            (
                                "Disk Usage",
                                format!("{:.2} GB / {:.2} GB", used_gb, total_gb),
                            ),
                            ("Usage Percentage", format!("{:.1}%", pct)),
                            (
                                "Status",
                                "HOST DISK RECOVERED (usage below 80%)".to_string(),
                            ),
                        ];

                        let html_body = crate::templates::render_html_report(
                            "Host Storage Recovery Notification",
                            &alert_reason,
                            "#10b981",
                            "#f0fdf4",
                            &meta,
                            None,
                        );

                        if let Err(e) = notifier.send_notification(
                            "recovery",
                            &subject,
                            &plain_body,
                            &html_body,
                        ) {
                            eprintln!("Failed to send disk space recovery email: {}", e);
                        }

                        notifier
                            .send_webhook_alerts(
                                "recovery",
                                "Host Server",
                                &alert_reason,
                                "Host Disk Trigger",
                                "",
                            )
                            .await;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_z_score() {
        let mut history = vec![10.0; 4];
        assert_eq!(calculate_z_score(&history, 20.0, 1.0), None);

        history.push(10.0);
        assert_eq!(calculate_z_score(&history, 20.0, 1.0), Some(10.0));

        let history_with_variance = vec![8.0, 12.0, 8.0, 12.0, 8.0, 12.0, 8.0, 12.0, 8.0, 12.0];
        assert_eq!(
            calculate_z_score(&history_with_variance, 16.0, 1.0),
            Some(3.0)
        );

        let history_stable = vec![10.0; 5];
        assert_eq!(calculate_z_score(&history_stable, 11.0, 0.2), Some(5.0));
    }
}
