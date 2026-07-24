use crate::config::Config;

pub async fn send_slack_alert(
    config: &Config,
    alert_type: &str,
    container_name: &str,
    alert_reason: &str,
    timestamp: &str,
    truncated_logs: &str,
) {
    if let Some(ref url) = config.slack_webhook {
        let mut allowed_slack = true;
        if let Some(ref allowed) = config.slack_alerts {
            allowed_slack = allowed.iter().any(|t| t == alert_type);
        }

        if allowed_slack {
            let payload = serde_json::json!({
                "text": format!(
                    "*Dockture Container Alert*\n*Container Name:* `{}`\n*Status/Event:* `{}`\n*Timestamp:* `{}`\n\n*Diagnostic Logs:*\n```\n{}\n```",
                    container_name, alert_reason, timestamp, truncated_logs
                )
            });

            let client = reqwest::Client::new();
            match client.post(url).json(&payload).send().await {
                Ok(_) => println!("Slack webhook notification sent."),
                Err(e) => eprintln!("Failed to send Slack webhook: {}", e),
            }
        }
    }
}
