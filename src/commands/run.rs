use crate::config::Config;
use crate::monitor::Monitor;
use std::process;

pub async fn run_daemon(config_path: Option<&str>) {
    let config = Config::load_or_exit(config_path);

    let monitor = Monitor::new(config);
    println!("Starting dockture monitor...");
    if let Err(e) = monitor.run().await {
        eprintln!("Monitor runtime error: {}", e);
        process::exit(1);
    }
}
