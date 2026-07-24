use crate::ServiceSubcommands;

pub fn handle_service_subcommand(subcommand: ServiceSubcommands) -> Result<(), String> {
    let home = std::env::var("HOME")
        .map_err(|_| "Could not read HOME environment variable".to_string())?;
    let systemd_dir = format!("{}/.config/systemd/user", home);
    let service_file_path = format!("{}/dockture.service", systemd_dir);

    match subcommand {
        ServiceSubcommands::Install => {
            let current_exe = std::env::current_exe()
                .map_err(|e| format!("Failed to find current executable path: {}", e))?;
            let current_exe_str = current_exe
                .to_str()
                .ok_or_else(|| "Current executable path is not valid UTF-8".to_string())?;

            std::fs::create_dir_all(&systemd_dir)
                .map_err(|e| format!("Failed to create systemd user directory: {}", e))?;

            let service_content = format!(
                "[Unit]\n\
                 Description=Dockture Docker Monitor Daemon\n\
                 After=docker.service\n\n\
                 [Service]\n\
                 ExecStart={} run\n\
                 Restart=always\n\
                 RestartSec=10\n\n\
                 [Install]\n\
                 WantedBy=default.target\n",
                current_exe_str
            );

            std::fs::write(&service_file_path, service_content)
                .map_err(|e| format!("Failed to write systemd service file: {}", e))?;

            println!(
                "Systemd user service file created at: {}",
                service_file_path
            );

            run_systemctl(&["daemon-reload"])?;
            run_systemctl(&["enable", "dockture"])?;

            println!("Service 'dockture' installed and enabled successfully.");
            println!("To start it now, run: dockture service start");
        }
        ServiceSubcommands::Uninstall => {
            let _ = run_systemctl(&["stop", "dockture"]);
            let _ = run_systemctl(&["disable", "dockture"]);

            if std::path::Path::new(&service_file_path).exists() {
                std::fs::remove_file(&service_file_path)
                    .map_err(|e| format!("Failed to remove systemd service file: {}", e))?;
                println!("Systemd user service file removed.");
            }

            let _ = run_systemctl(&["daemon-reload"]);
            println!("Service 'dockture' uninstalled successfully.");
        }
        ServiceSubcommands::Start => {
            run_systemctl(&["start", "dockture"])?;
            println!("Service 'dockture' started.");
        }
        ServiceSubcommands::Stop => {
            run_systemctl(&["stop", "dockture"])?;
            println!("Service 'dockture' stopped.");
        }
        ServiceSubcommands::Restart => {
            run_systemctl(&["restart", "dockture"])?;
            println!("Service 'dockture' restarted.");
        }
        ServiceSubcommands::Status => {
            let _ = std::process::Command::new("systemctl")
                .args(["--user", "status", "dockture"])
                .status();
        }
    }
    Ok(())
}

fn run_systemctl(args: &[&str]) -> Result<(), String> {
    let mut cmd = std::process::Command::new("systemctl");
    cmd.arg("--user");
    for arg in args {
        cmd.arg(arg);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run systemctl {:?}: {}", args, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("systemctl {:?} failed: {}", args, stderr.trim()));
    }
    Ok(())
}
