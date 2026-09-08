use crate::config::Config;
use crate::monitor::daily_reporter;
use crate::notifier::Notifier;

pub async fn run_test_report(custom_config_path: Option<&str>) {
    let config = Config::load_or_exit(custom_config_path);
    println!("Connecting to Docker daemon to generate daily summary report test...");

    let docker = match crate::utils::connect_docker() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to connect to Docker: {}", e);
            std::process::exit(1);
        }
    };

    let notifier = Notifier::new(config.clone());
    let stats = daily_reporter::new_shared_daily_stats();

    println!("Generating and sending test Daily Summary Report to {:?}...", config.receiver_emails);

    match daily_reporter::generate_and_send_daily_report(&docker, &config, &notifier, &stats).await {
        Ok(_) => {
            println!("SUCCESS: Daily Summary Report sent successfully!");
        }
        Err(e) => {
            eprintln!("ERROR: Failed to send Daily Summary Report: {}", e);
            std::process::exit(1);
        }
    }
}
