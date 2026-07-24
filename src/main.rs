#![allow(
    clippy::collapsible_if,
    clippy::collapsible_match,
    clippy::useless_format,
    clippy::manual_range_contains
)]

use clap::{CommandFactory, Parser, Subcommand};
use std::process;

mod commands;
mod config;
mod monitor;
mod notifier;
mod templates;
pub mod utils;

#[derive(Parser)]
#[command(name = "dockture")]
#[command(about = "A doctor for Docker containers. Monitors events (crashes, OOMs, unhealthy status) and dispatches alerts via SMTP.", long_about = None)]
struct Cli {
    #[arg(long, global = true, env = "DOCKTURE_CONFIG")]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Commands {
    Init,
    Run,
    TestEmail,
    TestWebhook,
    Config {
        #[command(subcommand)]
        subcommand: ConfigSubcommands,
    },
    Status,
    Logs {
        container: String,
        #[arg(long, default_value_t = 100)]
        tail: usize,
        #[arg(long)]
        follow: bool,
    },
    Complete {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    Service {
        #[command(subcommand)]
        subcommand: ServiceSubcommands,
    },
    Manual,
}

#[derive(Subcommand)]
pub enum ServiceSubcommands {
    Install,
    Uninstall,
    Start,
    Stop,
    Restart,
    Status,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ConfigSubcommands {
    Show,
    AddReceiver {
        email: String,
    },
    RemoveReceiver {
        email: String,
    },
    Set {
        #[arg(long)]
        smtp_host: Option<String>,
        #[arg(long)]
        smtp_port: Option<u16>,
        #[arg(long)]
        smtp_user: Option<String>,
        #[arg(long)]
        smtp_pass: Option<String>,
        #[arg(long)]
        sender_email: Option<String>,
        #[arg(long)]
        log_tail_size: Option<usize>,
        #[arg(long)]
        discord_webhook: Option<String>,
        #[arg(long)]
        slack_webhook: Option<String>,
        #[arg(long)]
        ignored_containers: Option<String>,
        #[arg(long)]
        monitored_containers: Option<String>,
        #[arg(long)]
        email_alerts: Option<String>,
        #[arg(long)]
        discord_alerts: Option<String>,
        #[arg(long)]
        slack_alerts: Option<String>,
        #[arg(long)]
        auto_restart: Option<bool>,
        #[arg(long)]
        log_keywords: Option<String>,
        #[arg(long)]
        anomaly_detection: Option<bool>,
        #[arg(long)]
        anomaly_threshold: Option<f64>,
        #[arg(long)]
        anomaly_sensitivity: Option<f64>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let cfg_path = cli.config.as_deref();

    match cli.command {
        Commands::Init => {
            commands::init::run_init();
        }
        Commands::Run => {
            commands::run::run_daemon(cfg_path).await;
        }
        Commands::TestEmail => {
            commands::test_email::run_test_email(cfg_path).await;
        }
        Commands::TestWebhook => {
            commands::test_webhook::run_test_webhook(cfg_path).await;
        }
        Commands::Config { subcommand } => {
            commands::config_mgmt::handle_config_command(subcommand, cfg_path);
        }
        Commands::Status => {
            if let Err(e) = commands::status::run_status().await {
                eprintln!("Error fetching status: {}", e);
                process::exit(1);
            }
        }
        Commands::Logs {
            container,
            tail,
            follow,
        } => {
            if let Err(e) = commands::logs::run_logs(&container, tail, follow, cfg_path).await {
                eprintln!("Error reading logs: {}", e);
                process::exit(1);
            }
        }
        Commands::Complete { shell } => {
            let mut cmd = Cli::command();
            let name = "dockture";
            clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
        Commands::Service { subcommand } => {
            if let Err(e) = commands::service::handle_service_subcommand(subcommand) {
                eprintln!("Service manager error: {}", e);
                process::exit(1);
            }
        }
        Commands::Manual => {
            if let Err(e) = commands::manual::run_manual() {
                eprintln!("Manual execution error: {}", e);
                process::exit(1);
            }
        }
    }
}
