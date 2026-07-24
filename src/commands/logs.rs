use crate::config::Config;
use bollard::container::LogsOptions;
use futures_util::StreamExt;

pub async fn run_logs(
    container: &str,
    tail: usize,
    follow: bool,
    config_path: Option<&str>,
) -> Result<(), String> {
    let docker = crate::utils::connect_docker()?;

    let config = Config::load_from_path(config_path);
    let keywords = match config {
        Ok(ref cfg) => cfg
            .log_keywords
            .clone()
            .unwrap_or_else(|| vec!["error".to_string(), "fatal".to_string(), "fail".to_string()]),
        Err(_) => vec!["error".to_string(), "fatal".to_string(), "fail".to_string()],
    };

    let log_options = LogsOptions::<String> {
        follow,
        stdout: true,
        stderr: true,
        tail: tail.to_string(),
        ..Default::default()
    };

    let mut stream = docker.logs(container, Some(log_options));

    println!(
        "\x1b[1;36m--- Streaming logs for '{}' (Highlighting: {}) ---\x1b[0m",
        container,
        keywords.join(", ")
    );

    while let Some(log_res) = stream.next().await {
        match log_res {
            Ok(output) => {
                let text = match output {
                    bollard::container::LogOutput::StdOut { message } => {
                        String::from_utf8_lossy(&message).into_owned()
                    }
                    bollard::container::LogOutput::StdErr { message } => {
                        String::from_utf8_lossy(&message).into_owned()
                    }
                    bollard::container::LogOutput::Console { message } => {
                        String::from_utf8_lossy(&message).into_owned()
                    }
                    _ => String::new(),
                };

                for line in text.lines() {
                    let mut highlighted_line = line.to_string();
                    let lower_line = line.to_lowercase();

                    let mut should_highlight = false;
                    for kw in &keywords {
                        if lower_line.contains(&kw.to_lowercase()) {
                            should_highlight = true;
                            break;
                        }
                    }

                    if should_highlight {
                        highlighted_line = format!("\x1b[1;31m{}\x1b[0m", line);
                    }

                    println!("{}", highlighted_line);
                }
            }
            Err(e) => return Err(format!("Error reading logs: {}", e)),
        }
    }

    Ok(())
}
