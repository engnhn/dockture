use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Cursor;
use std::path::{Path, PathBuf};
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

async fn download_asset(
    client: &reqwest::Client,
    url: &str,
    label: &str,
) -> Result<Vec<u8>, String> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download {}: {}", label, e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!(
            "Failed to download {}. HTTP status: {}",
            label, status
        ));
    }

    resp.bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|e| format!("Failed to read {} payload: {}", label, e))
}

fn parse_sha256_checksum(content: &[u8]) -> Result<String, String> {
    let text = std::str::from_utf8(content)
        .map_err(|e| format!("Checksum file is not valid UTF-8: {}", e))?;
    let checksum = text
        .split_whitespace()
        .next()
        .ok_or_else(|| "Checksum file is empty.".to_string())?
        .to_ascii_lowercase();

    if checksum.len() != 64 || !checksum.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Checksum file does not start with a valid SHA-256 digest.".to_string());
    }

    Ok(checksum)
}

fn verify_sha256(bytes: &[u8], expected: &str) -> Result<(), String> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual != expected {
        return Err(format!(
            "SHA-256 verification failed. Expected {}, got {}.",
            expected, actual
        ));
    }

    Ok(())
}

fn extract_dockture_binary(archive_bytes: &[u8], extract_dir: &Path) -> Result<PathBuf, String> {
    let decoder = flate2::read::GzDecoder::new(Cursor::new(archive_bytes));
    let mut archive = tar::Archive::new(decoder);
    let output_path = extract_dir.join("dockture");
    let canonical_extract_dir = extract_dir.canonicalize().map_err(|e| {
        format!(
            "Failed to canonicalize temporary extraction directory: {}",
            e
        )
    })?;

    for entry_res in archive
        .entries()
        .map_err(|e| format!("Failed to read update archive entries: {}", e))?
    {
        let mut entry = entry_res.map_err(|e| format!("Failed to read archive entry: {}", e))?;
        let entry_path = entry
            .path()
            .map_err(|e| format!("Failed to read archive entry path: {}", e))?;

        if entry_path == Path::new("dockture") {
            if !entry.header().entry_type().is_file() {
                return Err("Archive entry 'dockture' is not a regular file.".to_string());
            }

            entry
                .unpack(&output_path)
                .map_err(|e| format!("Failed to extract dockture binary: {}", e))?;

            let canonical_output = output_path
                .canonicalize()
                .map_err(|e| format!("Failed to canonicalize extracted binary path: {}", e))?;
            if !canonical_output.starts_with(&canonical_extract_dir) {
                let _ = fs::remove_file(&output_path);
                return Err(
                    "Archive attempted to extract outside the temporary directory.".to_string(),
                );
            }

            return Ok(output_path);
        }
    }

    Err("Extracted archive did not contain the 'dockture' binary executable.".to_string())
}

fn verify_and_extract_update(
    archive_bytes: &[u8],
    checksum_bytes: &[u8],
    extract_dir: &Path,
) -> Result<PathBuf, String> {
    let expected_checksum = parse_sha256_checksum(checksum_bytes)?;
    verify_sha256(archive_bytes, &expected_checksum)?;
    extract_dockture_binary(archive_bytes, extract_dir)
}

fn stage_binary_for_atomic_replace(
    extracted_bin_path: &Path,
    current_exe: &Path,
) -> Result<PathBuf, String> {
    let install_dir = current_exe
        .parent()
        .ok_or_else(|| "Current executable path has no parent directory.".to_string())?;
    let mut staged = tempfile::Builder::new()
        .prefix(".dockture-update-")
        .tempfile_in(install_dir)
        .map_err(|e| {
            format!(
                "Failed to create staged update file in install directory: {}",
                e
            )
        })?;

    {
        let mut src = File::open(extracted_bin_path)
            .map_err(|e| format!("Failed to open extracted binary: {}", e))?;
        let dst = staged.as_file_mut();
        std::io::copy(&mut src, dst)
            .map_err(|e| format!("Failed to stage update binary: {}", e))?;
        dst.sync_all()
            .map_err(|e| format!("Failed to sync staged update binary: {}", e))?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(staged.path(), fs::Permissions::from_mode(0o755)).map_err(|e| {
            format!(
                "Failed to set executable permissions on staged binary: {}",
                e
            )
        })?;
    }

    let (_file, path) = staged
        .keep()
        .map_err(|e| format!("Failed to persist staged update file: {}", e.error))?;

    Ok(path)
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
    let checksum_asset_name = format!("{}.sha256", asset.name);
    let checksum_asset = release
        .assets
        .iter()
        .find(|a| a.name == checksum_asset_name)
        .ok_or_else(|| {
            format!(
                "No SHA-256 checksum asset '{}' found in release {}",
                checksum_asset_name, latest_tag
            )
        })?;

    println!("Downloading update package: {}...", asset.name);

    let bytes = download_asset(&client, &asset.browser_download_url, "asset tarball").await?;
    let checksum_bytes = download_asset(
        &client,
        &checksum_asset.browser_download_url,
        "SHA-256 checksum",
    )
    .await?;
    println!("Verifying SHA-256 checksum...");
    let temp_dir = tempfile::Builder::new()
        .prefix("dockture-update-")
        .tempdir()
        .map_err(|e| format!("Failed to create temporary update directory: {}", e))?;

    println!("Extracting binary payload...");
    let extracted_bin_path = verify_and_extract_update(&bytes, &checksum_bytes, temp_dir.path())?;

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to determine current executable path: {}", e))?;

    println!("Replacing binary at '{}'...", current_exe.display());

    let staged_bin_path = stage_binary_for_atomic_replace(&extracted_bin_path, &current_exe)?;
    if let Err(e) = fs::rename(&staged_bin_path, &current_exe) {
        let _ = fs::remove_file(&staged_bin_path);
        return Err(format!(
            "Failed to atomically replace binary executable. Existing binary was left unchanged: {}",
            e
        ));
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;

    fn build_test_archive(entry_name: &str, body: &[u8]) -> Vec<u8> {
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        {
            let mut builder = tar::Builder::new(&mut gz);
            let mut header = tar::Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder
                .append_data(&mut header, entry_name, body)
                .expect("append test archive entry");
            builder.finish().expect("finish tar archive");
        }
        gz.finish().expect("finish gzip archive")
    }

    #[test]
    fn parses_sha256sum_style_checksum_file() {
        let checksum = parse_sha256_checksum(
            b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  dockture.tar.gz\n",
        )
        .expect("valid checksum");

        assert_eq!(
            checksum,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        );
    }

    #[test]
    fn rejects_invalid_checksum_file() {
        assert!(parse_sha256_checksum(b"not-a-checksum dockture.tar.gz").is_err());
    }

    #[test]
    fn verifies_matching_sha256() {
        let expected = format!("{:x}", Sha256::digest(b"dockture"));
        assert!(verify_sha256(b"dockture", &expected).is_ok());
        assert!(verify_sha256(b"tampered", &expected).is_err());
    }

    #[test]
    fn extracts_expected_binary_from_archive() {
        let archive = build_test_archive("dockture", b"binary");
        let temp_dir = tempfile::tempdir().expect("tempdir");

        let path = extract_dockture_binary(&archive, temp_dir.path()).expect("extract binary");
        let extracted = fs::read(path).expect("read extracted binary");

        assert_eq!(extracted, b"binary");
    }

    #[test]
    fn checksum_mismatch_stops_before_extraction() {
        let archive = build_test_archive("dockture", b"binary");
        let wrong_checksum =
            b"0000000000000000000000000000000000000000000000000000000000000000  dockture.tar.gz";
        let temp_dir = tempfile::tempdir().expect("tempdir");

        let err = verify_and_extract_update(&archive, wrong_checksum, temp_dir.path())
            .expect_err("mismatch");

        assert!(err.contains("SHA-256 verification failed"));
        assert!(!temp_dir.path().join("dockture").exists());
    }

    #[test]
    fn rejects_archive_without_expected_binary() {
        let archive = build_test_archive("other", b"binary");
        let temp_dir = tempfile::tempdir().expect("tempdir");

        assert!(extract_dockture_binary(&archive, temp_dir.path()).is_err());
    }
}
