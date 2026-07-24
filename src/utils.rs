pub async fn get_disk_usage(path: &str) -> Result<(u64, u64), String> {
    let output = tokio::process::Command::new("df")
        .args(["-P", "-B1", path])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("df command returned non-zero status".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() < 2 {
        return Err("Unexpected df output format".to_string());
    }

    let cols: Vec<&str> = lines[1].split_whitespace().collect();
    if cols.len() < 4 {
        return Err("Unexpected columns in df output".to_string());
    }

    let total: u64 = cols[1]
        .parse()
        .map_err(|e| format!("Failed to parse total blocks: {}", e))?;
    let used: u64 = cols[2]
        .parse()
        .map_err(|e| format!("Failed to parse used blocks: {}", e))?;

    Ok((used, total))
}

pub fn connect_docker() -> Result<bollard::Docker, String> {
    if let Ok(host) = std::env::var("DOCKER_HOST") {
        let trimmed = host.trim();
        if !trimmed.is_empty() {
            if let Some(socket_path) = trimmed.strip_prefix("unix://") {
                return bollard::Docker::connect_with_socket(
                    socket_path,
                    120,
                    bollard::API_DEFAULT_VERSION,
                )
                .map_err(|e| {
                    format!(
                        "Failed to connect to DOCKER_HOST socket '{}': {}",
                        trimmed, e
                    )
                });
            }
            if trimmed.starts_with("tcp://")
                || trimmed.starts_with("http://")
                || trimmed.starts_with("https://")
            {
                return bollard::Docker::connect_with_http_defaults().map_err(|e| {
                    format!(
                        "Failed to connect to DOCKER_HOST HTTP/TCP '{}': {}",
                        trimmed, e
                    )
                });
            }
        }
    }

    bollard::Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to local Docker daemon: {}", e))
}

pub fn matches_pattern(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.starts_with('*') && pattern.ends_with('*') {
        let clean = &pattern[1..pattern.len() - 1];
        name.contains(clean)
    } else if pattern.ends_with('*') {
        let clean = &pattern[0..pattern.len() - 1];
        name.starts_with(clean)
    } else if let Some(clean) = pattern.strip_prefix('*') {
        name.ends_with(clean)
    } else {
        name == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("app-web-1", "*"));
        assert!(matches_pattern("app-web-1", "app-*"));
        assert!(matches_pattern("app-web-1", "*-web-*"));
        assert!(matches_pattern("app-web-1", "*-1"));
        assert!(!matches_pattern("app-web-1", "db-*"));
        assert!(!matches_pattern("app-web-1", "*-web"));
        assert!(matches_pattern("db-postgres", "db-postgres"));
    }
}
