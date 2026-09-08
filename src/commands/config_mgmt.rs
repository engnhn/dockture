use crate::ConfigSubcommands;
use crate::config::Config;
use std::process;

pub fn handle_config_command(subcommand: ConfigSubcommands, config_path: Option<&str>) {
    let mut config = Config::load_or_exit(config_path);

    match subcommand {
        ConfigSubcommands::Show => {
            config.print_summary();
        }
        ConfigSubcommands::AddReceiver { email } => {
            if let Err(e) = config.add_receiver(email.clone()) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
            if let Err(e) = config.save_to_path(config_path) {
                eprintln!("Error saving config: {}", e);
                process::exit(1);
            }
            println!("Successfully added '{}' to receiver list.", email);
        }
        ConfigSubcommands::RemoveReceiver { email } => {
            if let Err(e) = config.remove_receiver(&email) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
            if let Err(e) = config.save_to_path(config_path) {
                eprintln!("Error saving config: {}", e);
                process::exit(1);
            }
            println!("Successfully removed '{}' from receiver list.", email);
        }
        ConfigSubcommands::Set {
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_pass,
            sender_email,
            log_tail_size,
            discord_webhook,
            slack_webhook,
            ignored_containers,
            monitored_containers,
            email_alerts,
            discord_alerts,
            slack_alerts,
            auto_restart,
            log_keywords,
            anomaly_detection,
            anomaly_threshold,
            anomaly_sensitivity,
            send_recovery_emails,
            alert_cooldown_secs,
            anomaly_min_value_cpu,
            anomaly_min_value_mem,
            anomaly_cooldown_secs,
            daily_report_enabled,
            daily_report_time,
            ignored_log_patterns,
        } => {
            let mut updated = false;
            if let Some(host) = smtp_host {
                config.smtp_host = host;
                updated = true;
            }
            if let Some(port) = smtp_port {
                config.smtp_port = port;
                updated = true;
            }
            if let Some(user) = smtp_user {
                config.smtp_user = user;
                updated = true;
            }
            if let Some(pass) = smtp_pass {
                config.smtp_pass = pass;
                updated = true;
            }
            if let Some(sender) = sender_email {
                config.sender_email = sender;
                updated = true;
            }
            if let Some(tail) = log_tail_size {
                config.log_tail_size = tail;
                updated = true;
            }
            if let Some(discord) = discord_webhook {
                config.discord_webhook = if discord.trim().is_empty() {
                    None
                } else {
                    Some(discord.trim().to_string())
                };
                updated = true;
            }
            if let Some(slack) = slack_webhook {
                config.slack_webhook = if slack.trim().is_empty() {
                    None
                } else {
                    Some(slack.trim().to_string())
                };
                updated = true;
            }
            if let Some(ignored) = ignored_containers {
                config.ignored_containers = if ignored.trim().is_empty() {
                    None
                } else {
                    Some(
                        ignored
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    )
                };
                updated = true;
            }
            if let Some(monitored) = monitored_containers {
                config.monitored_containers = if monitored.trim().is_empty() {
                    None
                } else {
                    Some(
                        monitored
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    )
                };
                updated = true;
            }
            if let Some(alerts) = email_alerts {
                config.email_alerts = if alerts.trim().is_empty() {
                    Some(vec![])
                } else if alerts.trim() == "all" || alerts.trim() == "default" {
                    None
                } else {
                    Some(
                        alerts
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    )
                };
                updated = true;
            }
            if let Some(alerts) = discord_alerts {
                config.discord_alerts = if alerts.trim().is_empty() {
                    Some(vec![])
                } else if alerts.trim() == "all" || alerts.trim() == "default" {
                    None
                } else {
                    Some(
                        alerts
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    )
                };
                updated = true;
            }
            if let Some(alerts) = slack_alerts {
                config.slack_alerts = if alerts.trim().is_empty() {
                    Some(vec![])
                } else if alerts.trim() == "all" || alerts.trim() == "default" {
                    None
                } else {
                    Some(
                        alerts
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    )
                };
                updated = true;
            }
            if let Some(restart) = auto_restart {
                config.auto_restart = Some(restart);
                updated = true;
            }
            if let Some(keywords) = log_keywords {
                config.log_keywords = if keywords.trim().is_empty() {
                    Some(vec![])
                } else {
                    Some(
                        keywords
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    )
                };
                updated = true;
            }
            if let Some(detect) = anomaly_detection {
                config.anomaly_detection = Some(detect);
                updated = true;
            }
            if let Some(threshold) = anomaly_threshold {
                config.anomaly_threshold = Some(threshold);
                updated = true;
            }
            if let Some(sensitivity) = anomaly_sensitivity {
                config.anomaly_sensitivity = Some(sensitivity);
                updated = true;
            }
            if let Some(recovery) = send_recovery_emails {
                config.send_recovery_emails = Some(recovery);
                updated = true;
            }
            if let Some(cooldown) = alert_cooldown_secs {
                config.alert_cooldown_secs = Some(cooldown);
                updated = true;
            }
            if let Some(min_cpu) = anomaly_min_value_cpu {
                config.anomaly_min_value_cpu = Some(min_cpu);
                updated = true;
            }
            if let Some(min_mem) = anomaly_min_value_mem {
                config.anomaly_min_value_mem = Some(min_mem);
                updated = true;
            }
            if let Some(anom_cd) = anomaly_cooldown_secs {
                config.anomaly_cooldown_secs = Some(anom_cd);
                updated = true;
            }
            if let Some(daily_enabled) = daily_report_enabled {
                config.daily_report_enabled = Some(daily_enabled);
                updated = true;
            }
            if let Some(daily_time) = daily_report_time {
                config.daily_report_time = Some(daily_time);
                updated = true;
            }
            if let Some(ignored_pats) = ignored_log_patterns {
                config.ignored_log_patterns = if ignored_pats.trim().is_empty() {
                    None
                } else {
                    Some(
                        ignored_pats
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    )
                };
                updated = true;
            }

            if updated {
                if let Err(e) = config.save_to_path(config_path) {
                    eprintln!("Error saving config: {}", e);
                    process::exit(1);
                }
                println!("Configuration updated successfully.");
            } else {
                println!("No changes provided to update.");
            }
        }
    }
}
