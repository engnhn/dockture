mod discord;
mod slack;
mod smtp;

use crate::config::Config;

#[derive(Clone)]
pub struct Notifier {
    config: Config,
}

impl Notifier {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn send_notification(
        &self,
        alert_type: &str,
        subject: &str,
        plain_body: &str,
        html_body: &str,
    ) -> Result<(), String> {
        smtp::send_email(&self.config, alert_type, subject, plain_body, html_body)
    }

    pub async fn send_webhook_alerts(
        &self,
        alert_type: &str,
        container_name: &str,
        alert_reason: &str,
        timestamp: &str,
        logs: &str,
    ) {
        let truncated_logs = crate::utils::truncate_str(logs, 1000);

        discord::send_discord_alert(
            &self.config,
            alert_type,
            container_name,
            alert_reason,
            timestamp,
            &truncated_logs,
        )
        .await;

        slack::send_slack_alert(
            &self.config,
            alert_type,
            container_name,
            alert_reason,
            timestamp,
            &truncated_logs,
        )
        .await;
    }
}
