use crate::config::Config;

pub async fn send_discord_alert(
    config: &Config,
    alert_type: &str,
    container_name: &str,
    alert_reason: &str,
    timestamp: &str,
    truncated_logs: &str,
) {
    if let Some(ref url) = config.discord_webhook {
        let mut allowed_discord = true;
        if let Some(ref allowed) = config.discord_alerts {
            allowed_discord = allowed.iter().any(|t| t == alert_type);
        }

        if allowed_discord {
            let color = if alert_reason.contains("OOM") {
                15671332
            } else if alert_reason.contains("UNHEALTHY") {
                14261766
            } else {
                14753112
            };

            let payload = serde_json::json!({
                "embeds": [{
                    "title": "[DOCKTURE] Container Alert",
                    "color": color,
                    "fields": [
                        { "name": "Container Name", "value": container_name, "inline": true },
                        { "name": "Status/Event", "value": alert_reason, "inline": true },
                        { "name": "Timestamp", "value": timestamp, "inline": false }
                    ],
                    "description": format!("**Diagnostic Logs:**\n```\n{}\n```", truncated_logs)
                }]
            });

            let client = reqwest::Client::new();
            match client.post(url).json(&payload).send().await {
                Ok(_) => println!("Discord webhook notification sent."),
                Err(e) => eprintln!("Failed to send Discord webhook: {}", e),
            }
        }
    }
}
