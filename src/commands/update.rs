use serde::Deserialize;
use std::fs;
use std::process::Command;

#[derive(Deserialize, Debug)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct ReleaseResponse {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

pub async fn run_update() -> Result<(), String> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Checking for latest Dockture release on GitHub...");

    let client = reqwest::Client::builder()
        .user_agent("dockture-update")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let release_url = "https://api.github.com/repos/engnhn/dockture/releases/latest";
    let resp = client
        .get(release_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch release info from GitHub: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "GitHub API returned HTTP status {}. Release lookup failed.",
            resp.status()
        ));
    }

    let release: ReleaseResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub release JSON response: {}", e))?;

    let latest_tag = release.tag_name.trim();
    let latest_ver = latest_tag.trim_start_matches('v');

    if latest_ver == current_version {
        println!(
            "Dockture is already up to date! Current version: v{}",
            current_version
        );
        return Ok(());
    }

    println!(
        "New release available: {} (Current: v{})",
        latest_tag, current_version
    );

    let target = if cfg!(target_arch = "x86_64") {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64-unknown-linux-gnu"
    } else {
        return Err(
            "Unsupported CPU architecture for automated binary update. Please build from source."
                .to_string(),
        );
    };

    let asset = release
        .assets
        .iter()
        .find(|a| a.name.contains(target) && a.name.ends_with(".tar.gz"))
        .ok_or_else(|| {
            format!(
                "No release asset found matching target architecture '{}' in release {}",
                target, latest_tag
            )
        })?;

    println!("Downloading update package: {}...", asset.name);

    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download asset tarball: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read asset payload: {}", e))?;

    let temp_dir = std::env::temp_dir();
    let tar_path = temp_dir.join("dockture_update.tar.gz");
    let extracted_bin_path = temp_dir.join("dockture");

    fs::write(&tar_path, &bytes)
        .map_err(|e| format!("Failed to write temporary update tarball: {}", e))?;

    println!("Extracting binary payload...");
    let tar_status = Command::new("tar")
        .args([
            "-xzf",
            tar_path.to_str().unwrap_or_default(),
            "-C",
            temp_dir.to_str().unwrap_or_default(),
        ])
        .status()
        .map_err(|e| format!("Failed to execute 'tar' command: {}", e))?;

    if !tar_status.success() {
        let _ = fs::remove_file(&tar_path);
        return Err("Extraction failed. 'tar' command returned a non-zero exit code.".to_string());
    }

    if !extracted_bin_path.exists() {
        let _ = fs::remove_file(&tar_path);
        return Err(
            "Extracted archive did not contain the 'dockture' binary executable.".to_string(),
        );
    }

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to determine current executable path: {}", e))?;

    println!("Replacing binary at '{}'...", current_exe.display());

    // Set executable permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&extracted_bin_path, fs::Permissions::from_mode(0o755));
    }

    // Try rename first, fallback to copy
    if let Err(e) = fs::rename(&extracted_bin_path, &current_exe) {
        fs::copy(&extracted_bin_path, &current_exe).map_err(|copy_err| {
            format!(
                "Failed to replace binary executable (rename err: {}, copy err: {})",
                e, copy_err
            )
        })?;
        let _ = fs::remove_file(&extracted_bin_path);
    }

    let _ = fs::remove_file(&tar_path);

    println!("SUCCESS: Dockture updated successfully to {}!", latest_tag);

    // Restart service if installed
    if let Ok(home) = std::env::var("HOME") {
        let service_file = format!("{}/.config/systemd/user/dockture.service", home);
        if std::path::Path::new(&service_file).exists() {
            println!("Restarting active Dockture systemd service...");
            let status = Command::new("systemctl")
                .args(["--user", "restart", "dockture"])
                .status();
            if let Ok(st) = status {
                if st.success() {
                    println!("Systemd service 'dockture' restarted successfully.");
                }
            }
        }
    }

    Ok(())
}
