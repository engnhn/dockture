use dialoguer::{Input, Password};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub sender_email: String,
    pub receiver_emails: Vec<String>,
    pub log_tail_size: usize,
    #[serde(default)]
    pub ignored_containers: Option<Vec<String>>,
    #[serde(default)]
    pub monitored_containers: Option<Vec<String>>,
    #[serde(default)]
    pub discord_webhook: Option<String>,
    #[serde(default)]
    pub slack_webhook: Option<String>,
    #[serde(default)]
    pub email_alerts: Option<Vec<String>>,
    #[serde(default)]
    pub discord_alerts: Option<Vec<String>>,
    #[serde(default)]
    pub slack_alerts: Option<Vec<String>>,
    #[serde(default)]
    pub auto_restart: Option<bool>,
    #[serde(default)]
    pub log_keywords: Option<Vec<String>>,
    #[serde(default)]
    pub anomaly_detection: Option<bool>,
    #[serde(default)]
    pub anomaly_threshold: Option<f64>,
    #[serde(default)]
    pub anomaly_sensitivity: Option<f64>,
}

impl Config {
    pub fn anomaly_detection(&self) -> bool {
        self.anomaly_detection.unwrap_or(true)
    }

    pub fn anomaly_threshold(&self) -> f64 {
        self.anomaly_threshold.unwrap_or(3.0)
    }

    pub fn anomaly_sensitivity(&self) -> f64 {
        self.anomaly_sensitivity.unwrap_or(0.2)
    }

    pub fn is_container_monitored(&self, container_name: &str) -> bool {
        if let Some(ref ignored) = self.ignored_containers {
            if ignored
                .iter()
                .any(|pat| crate::utils::matches_pattern(container_name, pat))
            {
                return false;
            }
        }
        if let Some(ref monitored) = self.monitored_containers {
            if !monitored.is_empty()
                && !monitored
                    .iter()
                    .any(|pat| crate::utils::matches_pattern(container_name, pat))
            {
                return false;
            }
        }
        true
    }
    pub fn default_path() -> Result<PathBuf, String> {
        let home = std::env::var("HOME").map_err(|_| {
            "HOME environment variable not set. Cannot determine config path.".to_string()
        })?;
        Ok(PathBuf::from(home)
            .join(".config")
            .join("dockture")
            .join("config.toml"))
    }

    pub fn resolve_path(custom_path: Option<&str>) -> Result<PathBuf, String> {
        if let Some(p) = custom_path {
            if !p.trim().is_empty() {
                return Ok(PathBuf::from(p));
            }
        }
        if let Ok(env_path) = std::env::var("DOCKTURE_CONFIG") {
            if !env_path.trim().is_empty() {
                return Ok(PathBuf::from(env_path));
            }
        }
        Self::default_path()
    }

    pub fn load_from_path(custom_path: Option<&str>) -> Result<Self, String> {
        let path = Self::resolve_path(custom_path)?;
        if !path.exists() {
            return Err(format!(
                "Configuration file not found at {:?}. Please run 'dockture init' or specify --config / DOCKTURE_CONFIG.",
                path
            ));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file at {:?}: {}", path, e))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config TOML at {:?}: {}", path, e))?;

        Ok(config)
    }

    pub fn load_or_exit(custom_path: Option<&str>) -> Self {
        match Self::load_from_path(custom_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    #[allow(dead_code)]
    pub fn load() -> Result<Self, String> {
        Self::load_from_path(None)
    }

    pub fn save_to_path(&self, custom_path: Option<&str>) -> Result<(), String> {
        let path = Self::resolve_path(custom_path)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config to TOML: {}", e))?;

        fs::write(&path, content)
            .map_err(|e| format!("Failed to write config file to {:?}: {}", path, e))?;

        let mut perms = fs::metadata(&path)
            .map_err(|e| format!("Failed to read metadata of config: {}", e))?
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)
            .map_err(|e| format!("Failed to set strict permissions on config file: {}", e))?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), String> {
        self.save_to_path(None)
    }

    pub fn interactive_wizard() -> Result<Self, String> {
        println!("=== Dockture Configuration Wizard ===");
        println!("This wizard will guide you to configure SMTP and target emails for alerts.\n");

        let smtp_host: String = Input::new()
            .with_prompt("SMTP Host (e.g. smtp.gmail.com)")
            .interact_text()
            .map_err(|e| e.to_string())?;

        let smtp_port: u16 = Input::new()
            .with_prompt("SMTP Port (e.g. 587 or 465)")
            .default(587)
            .interact_text()
            .map_err(|e| e.to_string())?;

        let smtp_user: String = Input::new()
            .with_prompt("SMTP Username/Email")
            .interact_text()
            .map_err(|e| e.to_string())?;

        let smtp_pass = Password::new()
            .with_prompt("SMTP Password")
            .interact()
            .map_err(|e| e.to_string())?;

        let sender_email: String = Input::new()
            .with_prompt("Sender Email Address")
            .default(smtp_user.clone())
            .interact_text()
            .map_err(|e| e.to_string())?;

        let receiver_input: String = Input::new()
            .with_prompt("Receiver Email Addresses (comma-separated)")
            .interact_text()
            .map_err(|e| e.to_string())?;

        let receiver_emails: Vec<String> = receiver_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if receiver_emails.is_empty() {
            return Err("At least one receiver email is required.".to_string());
        }

        let log_tail_size: usize = Input::new()
            .with_prompt("Log tail lines to attach on alert")
            .default(50)
            .interact_text()
            .map_err(|e| e.to_string())?;

        let discord_input: String = Input::new()
            .with_prompt("Discord Webhook URL (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()
            .map_err(|e| e.to_string())?;

        let discord_webhook = if discord_input.trim().is_empty() {
            None
        } else {
            Some(discord_input.trim().to_string())
        };

        let slack_input: String = Input::new()
            .with_prompt("Slack Webhook URL (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()
            .map_err(|e| e.to_string())?;

        let slack_webhook = if slack_input.trim().is_empty() {
            None
        } else {
            Some(slack_input.trim().to_string())
        };

        let config = Config {
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_pass,
            sender_email,
            receiver_emails,
            log_tail_size,
            ignored_containers: None,
            monitored_containers: None,
            discord_webhook,
            slack_webhook,
            email_alerts: None,
            discord_alerts: None,
            slack_alerts: None,
            auto_restart: None,
            log_keywords: None,
            anomaly_detection: None,
            anomaly_threshold: None,
            anomaly_sensitivity: None,
        };

        Ok(config)
    }

    pub fn print_summary(&self) {
        println!("┌────────────────────────────────────────────────────────┐");
        println!("│              DOCKTURE CONFIGURATION SUMMARY            │");
        println!("├────────────────────────┬───────────────────────────────┤");
        println!("│ SMTP Host              │ {:<29} │", self.smtp_host);
        println!("│ SMTP Port              │ {:<29} │", self.smtp_port);
        println!("│ SMTP User              │ {:<29} │", self.smtp_user);
        println!("│ SMTP Password          │ ***************************** │");
        println!("│ Sender Email           │ {:<29} │", self.sender_email);
        println!("│ Log Tail Size          │ {:<29} │", self.log_tail_size);
        println!(
            "│ Discord Webhook        │ {:<29} │",
            if self.discord_webhook.is_some() {
                "Configured"
            } else {
                "Not Configured"
            }
        );
        println!(
            "│ Slack Webhook          │ {:<29} │",
            if self.slack_webhook.is_some() {
                "Configured"
            } else {
                "Not Configured"
            }
        );
        println!(
            "│ Auto Restart           │ {:<29} │",
            if self.auto_restart.unwrap_or(false) {
                "Enabled"
            } else {
                "Disabled"
            }
        );
        println!(
            "│ Log Keywords           │ {:<29} │",
            match &self.log_keywords {
                Some(kw) if kw.is_empty() => "Disabled".to_string(),
                Some(kw) => kw.join(", "),
                None => "Default (error, fatal, fail)".to_string(),
            }
        );
        println!(
            "│ Anomaly Detection      │ {:<29} │",
            if self.anomaly_detection() {
                "Enabled"
            } else {
                "Disabled"
            }
        );
        println!(
            "│ Anomaly Threshold      │ {:<29} │",
            format!("{:.1} std dev", self.anomaly_threshold())
        );
        println!(
            "│ Anomaly Sensitivity    │ {:<29} │",
            format!("{} min std dev", self.anomaly_sensitivity())
        );
        println!("├────────────────────────┴───────────────────────────────┤");
        println!("│ Receiver Emails:                                       │");
        for email in &self.receiver_emails {
            println!("│   - {:<50} │", email);
        }
        if let Some(ref ignored) = self.ignored_containers {
            if !ignored.is_empty() {
                println!("├────────────────────────────────────────────────────────┤");
                println!("│ Ignored Containers (Blacklist):                        │");
                for pattern in ignored {
                    println!("│   - {:<50} │", pattern);
                }
            }
        }
        if let Some(ref monitored) = self.monitored_containers {
            if !monitored.is_empty() {
                println!("├────────────────────────────────────────────────────────┤");
                println!("│ Monitored Containers (Whitelist):                      │");
                for pattern in monitored {
                    println!("│   - {:<50} │", pattern);
                }
            }
        }
        if let Some(ref email_a) = self.email_alerts {
            if !email_a.is_empty() {
                println!("├────────────────────────────────────────────────────────┤");
                println!("│ Email Routed Alerts:                                   │");
                println!("│   - {:<50} │", email_a.join(", "));
            }
        }
        if let Some(ref discord_a) = self.discord_alerts {
            if !discord_a.is_empty() {
                println!("├────────────────────────────────────────────────────────┤");
                println!("│ Discord Routed Alerts:                                 │");
                println!("│   - {:<50} │", discord_a.join(", "));
            }
        }
        if let Some(ref slack_a) = self.slack_alerts {
            if !slack_a.is_empty() {
                println!("├────────────────────────────────────────────────────────┤");
                println!("│ Slack Routed Alerts:                                   │");
                println!("│   - {:<50} │", slack_a.join(", "));
            }
        }
        println!("└────────────────────────────────────────────────────────┘");
    }

    pub fn add_receiver(&mut self, email: String) -> Result<(), String> {
        if self.receiver_emails.contains(&email) {
            return Err(format!(
                "Email '{}' is already in the receiver list.",
                email
            ));
        }
        if email.trim().is_empty() || !email.contains('@') {
            return Err(format!("Invalid email format: '{}'", email));
        }
        self.receiver_emails.push(email);
        Ok(())
    }

    pub fn remove_receiver(&mut self, email: &str) -> Result<(), String> {
        let original_len = self.receiver_emails.len();
        self.receiver_emails.retain(|e| e != email);
        if self.receiver_emails.len() == original_len {
            return Err(format!(
                "Email '{}' was not found in the receiver list.",
                email
            ));
        }
        if self.receiver_emails.is_empty() {
            return Err(
                "Cannot remove email: At least one receiver email is required.".to_string(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = Config {
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_user: "user".to_string(),
            smtp_pass: "pass".to_string(),
            sender_email: "sender@example.com".to_string(),
            receiver_emails: vec![
                "rec1@example.com".to_string(),
                "rec2@example.com".to_string(),
            ],
            log_tail_size: 100,
            ignored_containers: Some(vec!["test-*".to_string()]),
            monitored_containers: None,
            discord_webhook: Some("https://discord.com/api/webhooks/123".to_string()),
            slack_webhook: None,
            email_alerts: Some(vec!["crash".to_string(), "health".to_string()]),
            discord_alerts: None,
            slack_alerts: None,
            auto_restart: Some(true),
            log_keywords: Some(vec!["err".to_string()]),
            anomaly_detection: Some(true),
            anomaly_threshold: Some(3.0),
            anomaly_sensitivity: Some(0.2),
        };

        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.smtp_host, "smtp.example.com");
        assert_eq!(deserialized.smtp_port, 587);
        assert_eq!(deserialized.smtp_user, "user");
        assert_eq!(deserialized.smtp_pass, "pass");
        assert_eq!(deserialized.sender_email, "sender@example.com");
        assert_eq!(deserialized.receiver_emails.len(), 2);
        assert_eq!(deserialized.receiver_emails[0], "rec1@example.com");
        assert_eq!(deserialized.receiver_emails[1], "rec2@example.com");
        assert_eq!(deserialized.log_tail_size, 100);
        assert_eq!(deserialized.email_alerts.unwrap()[0], "crash");
    }
}
