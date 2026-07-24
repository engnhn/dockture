use bollard::Docker;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type RestartTracker = Arc<Mutex<HashMap<String, Vec<std::time::Instant>>>>;

pub struct SelfHealingResult {
    pub status: String,
    pub is_crash_loop: bool,
    pub updated_reason: String,
}

pub async fn handle_self_healing(
    docker: &Docker,
    container_id: &str,
    container_name: &str,
    initial_reason: &str,
    auto_restart_enabled: bool,
    tracker_arc: &RestartTracker,
) -> SelfHealingResult {
    let mut self_healing_status = "Not Enabled".to_string();
    let mut is_crash_loop = false;
    let mut alert_reason = initial_reason.to_string();

    if auto_restart_enabled {
        let now = std::time::Instant::now();
        let mut tracker = tracker_arc.lock().await;
        let history = tracker
            .entry(container_name.to_string())
            .or_insert_with(Vec::new);

        history.retain(|t| now.duration_since(*t) < std::time::Duration::from_secs(300));

        if history.len() >= 3 {
            is_crash_loop = true;
            self_healing_status = "Suspended (CrashLoopBackOff Detected)".to_string();
            alert_reason = format!(
                "{} [CRITICAL: CrashLoopBackOff Detected ({} restarts in 5m)]",
                alert_reason,
                history.len()
            );
            eprintln!(
                "Self-healing: Suspended auto-restart for '{}' due to CrashLoopBackOff limit ({} restarts in 5m)",
                container_name,
                history.len()
            );
        } else {
            history.push(now);
            println!(
                "Self-healing: Restarting container '{}' ({})",
                container_name, container_id
            );
            match docker.restart_container(container_id, None).await {
                Ok(_) => {
                    self_healing_status = "Success (Container Restarted)".to_string();
                    println!(
                        "Self-healing: Successfully restarted container '{}'",
                        container_name
                    );
                }
                Err(e) => {
                    self_healing_status = format!("Failed to restart: {}", e);
                    eprintln!(
                        "Self-healing: Failed to restart container '{}': {}",
                        container_name, e
                    );
                }
            }
        }
    }

    SelfHealingResult {
        status: self_healing_status,
        is_crash_loop,
        updated_reason: alert_reason,
    }
}
