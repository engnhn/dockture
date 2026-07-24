use crate::config::Config;
use crate::notifier::Notifier;
use std::process;

pub async fn run_test_webhook(config_path: Option<&str>) {
    let config = Config::load_or_exit(config_path);

    let has_discord = config
        .discord_webhook
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let has_slack = config
        .slack_webhook
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    if !has_discord && !has_slack {
        eprintln!("No Discord or Slack webhooks configured.");
        eprintln!(
            "Use 'dockture config set --discord-webhook <URL>' or '--slack-webhook <URL>' to set up webhooks."
        );
        process::exit(1);
    }

    if has_discord {
        println!("Dispatching test notification to Discord Webhook...");
    }
    if has_slack {
        println!("Dispatching test notification to Slack Webhook...");
    }

    let notifier = Notifier::new(config);
    let timestamp = "2026-07-24 20:53:00";

    notifier
        .send_webhook_alerts(
            "test",
            "dockture-test-container",
            "Webhook Setup Verification",
            timestamp,
            "[INFO] Dockture webhook diagnostic check executed successfully.",
        )
        .await;

    println!("Test webhook payloads dispatched successfully.");
}
