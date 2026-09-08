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

pub fn truncate_str(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while !s.is_char_boundary(end) && end > 0 {
        end -= 1;
    }
    format!("{}... [truncated]", &s[..end])
}

pub fn is_valid_keyword_match(line: &str, keyword: &str) -> bool {
    if keyword.is_empty() || line.is_empty() {
        return false;
    }

    let lower_line = line.to_lowercase();
    let lower_kw = keyword.to_lowercase();

    let mut start_search = 0;
    while let Some(idx) = lower_line[start_search..].find(&lower_kw) {
        let match_idx = start_search + idx;
        let match_end = match_idx + lower_kw.len();
        start_search = match_end;

        let char_before = if match_idx > 0 {
            lower_line[..match_idx].chars().next_back()
        } else {
            None
        };

        let char_after = if match_end < lower_line.len() {
            lower_line[match_end..].chars().next()
        } else {
            None
        };

        let is_prefix_boundary = match char_before {
            Some(c) => !c.is_alphanumeric(),
            None => true,
        };

        let is_suffix_boundary = match char_after {
            Some(c) => !c.is_alphanumeric(),
            None => true,
        };

        if !is_prefix_boundary || !is_suffix_boundary {
            continue;
        }

        let snippet_after = lower_line[match_end..lower_line.len().min(match_end + 40)]
            .trim_start_matches(|c: char| c == '"' || c == '\'' || c == ' ' || c == ':' || c == '=');

        if snippet_after.starts_with("false")
            || snippet_after.starts_with("null")
            || snippet_after.starts_with("0,")
            || snippet_after.starts_with("0}")
            || snippet_after.starts_with("0\n")
            || snippet_after.starts_with("0\r")
            || snippet_after.starts_with("\"\"")
            || snippet_after.starts_with("''")
        {
            continue;
        }

        return true;
    }

    false
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

    #[test]
    fn test_truncate_str_ascii() {
        let text = "Hello world from dockture";
        assert_eq!(truncate_str(text, 50), "Hello world from dockture");
        assert_eq!(truncate_str(text, 5), "Hello... [truncated]");
    }

    #[test]
    fn test_truncate_str_utf8_char_boundary() {
        let text = "Hata: 🚀 Sistem durduruldu 💥";
        // '🚀' is 4 bytes at index 7..11. If max_bytes is 9, it lands inside '🚀'.
        let truncated = truncate_str(text, 9);
        assert_eq!(truncated, "Hata: ... [truncated]");
        assert!(truncated.starts_with("Hata: "));
    }

    #[test]
    fn test_is_valid_keyword_match() {
        assert!(!is_valid_keyword_match("\"isCritical\": false,", "CRITICAL"));
        assert!(!is_valid_keyword_match("\"critical\": false,", "CRITICAL"));
        assert!(!is_valid_keyword_match("\"error\": null,", "ERROR"));
        assert!(!is_valid_keyword_match("\"errorCount\": 0", "ERROR"));
        assert!(!is_valid_keyword_match("uncritical_system = false", "CRITICAL"));
        assert!(!is_valid_keyword_match("\"failed\": false", "FAIL"));

        assert!(is_valid_keyword_match("[CRITICAL] Server crashed", "CRITICAL"));
        assert!(is_valid_keyword_match("fatal error occurred in worker", "FATAL"));
        assert!(is_valid_keyword_match("\"error\": \"Database connection timeout\"", "ERROR"));
        assert!(is_valid_keyword_match("status: CRITICAL_FAILURE", "CRITICAL"));
    }
}
