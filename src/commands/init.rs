use crate::config::Config;
use std::process;

pub fn run_init() {
    match Config::interactive_wizard() {
        Ok(config) => {
            if let Err(e) = config.save() {
                eprintln!("Error saving configuration: {}", e);
                process::exit(1);
            }
            println!("Configuration saved successfully!");
            if let Ok(path) = Config::default_path() {
                println!("Saved to: {:?}", path);
            }
        }
        Err(e) => {
            eprintln!("Configuration wizard failed: {}", e);
            process::exit(1);
        }
    }
}
