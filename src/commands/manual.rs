use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::Stylize,
    terminal::{self, ClearType},
};
use std::io::{Write, stdout};

struct Chapter {
    title: &'static str,
    content: &'static [&'static str],
}

pub fn run_manual() -> Result<(), String> {
    let chapters = get_chapters();
    let mut current_chapter = 0;
    let mut scroll_y = 0;

    let mut stdout = stdout();
    terminal::enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)
        .map_err(|e| format!("Failed to enter alternate screen: {}", e))?;

    let result = run_loop(&mut stdout, &chapters, &mut current_chapter, &mut scroll_y);

    let _ = execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();

    result
}

fn get_chapters() -> Vec<Chapter> {
    vec![
        Chapter {
            title: "1. Overview & Core Architecture",
            content: &[
                "Dockture is an asynchronous, event-driven container monitoring and",
                "self-healing daemon built in Rust. It connects to the Docker socket",
                "to stream lifecycle events, calculate Z-score anomalies, and dispatch alerts.",
                "",
                " Key Capabilities:",
                " ─────────────────",
                "  • Self-Healing: Automatically restarts crashed or unhealthy containers.",
                "  • CrashLoopBackOff Prevention: Suspends auto-restarts if a container restarts",
                "    >= 3 times within 5 minutes to prevent system resource exhaustion.",
                "  • Statistical Anomaly Detection: Analyzes CPU and RAM metrics using rolling",
                "    standard deviations (Z-score) to detect resource leaks before crashes occur.",
                "  • Flexible Alerting: Multi-channel notifications across SMTP Email,",
                "    Discord Webhooks, and Slack Webhooks with event-type routing.",
            ],
        },
        Chapter {
            title: "2. Installation & Setup Wizard",
            content: &[
                "To initialize Dockture for the first time, run the interactive setup wizard.",
                "",
                " Command:",
                " ────────",
                "  $ dockture init",
                "",
                " Wizard Steps:",
                " ─────────────",
                "  1. SMTP Server Host (e.g. smtp.gmail.com or smtp.office365.com)",
                "  2. SMTP Server Port (587 for TLS, 465 for SSL)",
                "  3. SMTP Username & Password (Input is masked automatically)",
                "  4. Sender Email Address",
                "  5. Recipient Email Addresses (Comma-separated for multiple receivers)",
                "  6. Failure Log Extraction Buffer Size (Default: 100 lines)",
                "",
                " Configuration Path & Permissions:",
                " ──────────────────────────────────",
                "  Settings are stored in ~/.config/dockture/config.toml with strict POSIX",
                "  0600 permissions (owner read/write only) to protect confidential keys.",
            ],
        },
        Chapter {
            title: "3. Configuration Management (CLI)",
            content: &[
                "You can inspect and update configuration parameters directly via CLI.",
                "",
                " Inspect Active Settings:",
                " ───────────────────────",
                "  $ dockture config show",
                "  (SMTP passwords are automatically masked for security)",
                "",
                " Recipient Email Management:",
                " ──────────────────────────",
                "  $ dockture config add-receiver admin@company.com",
                "  $ dockture config remove-receiver old@company.com",
                "",
                " Fine-Tuning Flags (Set Flags):",
                " ─────────────────────────────",
                "  • Webhook Integration:",
                "    $ dockture config set --discord-webhook \"https://discord.com/api/webhooks/...\"",
                "    $ dockture config set --slack-webhook \"https://hooks.slack.com/services/...\"",
                "",
                "  • Container Filtering (Glob Patterns):",
                "    $ dockture config set --monitored-containers \"prod-*,db-*\"",
                "    $ dockture config set --ignored-containers \"test-*,temp-*\"",
                "",
                "  • Auto-Restart and Anomaly Thresholds:",
                "    $ dockture config set --auto-restart true",
                "    $ dockture config set --anomaly-detection true --anomaly-threshold 3.5",
            ],
        },
        Chapter {
            title: "4. Live Status & Log Monitoring",
            content: &[
                "Dockture provides real-time terminal status dashboards and log streaming.",
                "",
                " Container Status Dashboard:",
                " ───────────────────────────",
                "  $ dockture status",
                "  Renders a table detailing Container ID, Name, Image, State, and",
                "  Health status alongside overall host disk space usage.",
                "",
                " Real-Time Log Streaming:",
                " ────────────────────────",
                "  $ dockture logs <container-name> --tail 100 --follow",
                "  Streams live container logs while highlighting key search signatures",
                "  (e.g., 'error', 'fatal', 'exception', 'panic') in bold red.",
            ],
        },
        Chapter {
            title: "5. Notification Routing & Categories",
            content: &[
                "Route specific notification types to different communication channels.",
                "",
                " Event Categories:",
                " ──────────────────",
                "  • crash   : Container crashes, non-zero exit codes, OOM terminations.",
                "  • health  : Unhealthy status transitions from Docker healthchecks.",
                "  • warning : Resource anomalies (CPU/RAM spikes) or log keyword matches.",
                "  • recovery: Containers returning to normal healthy state.",
                "",
                " Channel Routing Examples:",
                " ─────────────────────────",
                "  • Send only critical crashes to Email:",
                "    $ dockture config set --email-alerts \"crash\"",
                "",
                "  • Send crashes and warnings to Discord:",
                "    $ dockture config set --discord-alerts \"crash,warning\"",
                "",
                "  • Send all events to Slack:",
                "    $ dockture config set --slack-alerts \"crash,warning,health,recovery\"",
            ],
        },
        Chapter {
            title: "6. Systemd User Service Integration",
            content: &[
                "Run Dockture continuously as an unprivileged systemd user service.",
                "",
                " Service Installation & Startup:",
                " ────────────────────────────────",
                "  $ dockture service install",
                "  (Generates ~/.config/systemd/user/dockture.service automatically)",
                "",
                "  $ dockture service start",
                "  (Starts the daemon in the background)",
                "",
                " Service Lifecycle Management:",
                " ─────────────────────────────",
                "  $ dockture service status   (Query service execution status)",
                "  $ dockture service restart  (Restart background daemon)",
                "  $ dockture service stop     (Stop background daemon)",
                "  $ dockture service uninstall(Remove systemd unit file and registration)",
            ],
        },
        Chapter {
            title: "7. Diagnostic Testing & Verification",
            content: &[
                "Verify your SMTP host and Webhook settings before starting the daemon.",
                "",
                " SMTP Email Verification:",
                " ────────────────────────",
                "  $ dockture test-email",
                "  Connects to configured SMTP server and sends a test email to receivers.",
                "",
                " Webhook Payload Verification:",
                " ─────────────────────────────",
                "  $ dockture test-webhook",
                "  Transmits diagnostic test cards to Discord and Slack webhook URLs.",
            ],
        },
        Chapter {
            title: "8. Advanced Flags & Environment Variables",
            content: &[
                "Dockture supports custom config paths, remote Docker sockets, and completions.",
                "",
                " Custom Config Path (--config & DOCKTURE_CONFIG):",
                " ───────────────────────────────────────────────",
                "  $ dockture --config /etc/dockture/prod.toml run",
                "  $ export DOCKTURE_CONFIG=/etc/dockture/prod.toml",
                "",
                " Remote Docker Socket (DOCKER_HOST):",
                " ───────────────────────────────────",
                "  $ export DOCKER_HOST=tcp://192.168.1.100:2375",
                "  $ dockture status",
                "",
                " Shell Autocompletion Generation (Bash/Zsh/Fish):",
                " ───────────────────────────────────────────────",
                "  $ dockture complete bash > ~/.local/share/bash-completion/completions/dockture",
                "  $ dockture complete zsh > ~/.zsh/completion/_dockture",
            ],
        },
    ]
}

fn run_loop(
    stdout: &mut std::io::Stdout,
    chapters: &[Chapter],
    current_chapter: &mut usize,
    scroll_y: &mut usize,
) -> Result<(), String> {
    loop {
        let (width, height) =
            terminal::size().map_err(|e| format!("Failed to get terminal size: {}", e))?;
        let width = width as usize;
        let height = height as usize;

        if width < 40 || height < 10 {
            execute!(
                stdout,
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            )
            .unwrap();
            println!("Terminal window too small. Please enlarge your terminal.");
            stdout.flush().unwrap();

            if let Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('q') | KeyCode::Esc,
                ..
            })) = event::read()
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        let chapter = &chapters[*current_chapter];
        let content_lines = chapter.content;
        let max_scroll = if content_lines.len() + 6 > height {
            (content_lines.len() + 6) - height
        } else {
            0
        };

        if *scroll_y > max_scroll {
            *scroll_y = max_scroll;
        }

        render_screen(stdout, chapters, *current_chapter, *scroll_y, width, height)?;

        if let Ok(Event::Key(key_event)) = event::read() {
            match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Right | KeyCode::Char('n') | KeyCode::Char('l') => {
                    if *current_chapter < chapters.len() - 1 {
                        *current_chapter += 1;
                        *scroll_y = 0;
                    }
                }
                KeyCode::Left | KeyCode::Char('p') | KeyCode::Char('h') => {
                    if *current_chapter > 0 {
                        *current_chapter -= 1;
                        *scroll_y = 0;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if *scroll_y < max_scroll {
                        *scroll_y += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if *scroll_y > 0 {
                        *scroll_y -= 1;
                    }
                }
                KeyCode::PageDown | KeyCode::Char(' ') => {
                    let page_size = height.saturating_sub(4);
                    *scroll_y = std::cmp::min(*scroll_y + page_size, max_scroll);
                }
                KeyCode::PageUp => {
                    let page_size = height.saturating_sub(4);
                    *scroll_y = scroll_y.saturating_sub(page_size);
                }
                KeyCode::Home => {
                    *scroll_y = 0;
                }
                KeyCode::End => {
                    *scroll_y = max_scroll;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn render_screen(
    stdout: &mut std::io::Stdout,
    chapters: &[Chapter],
    current_chapter: usize,
    scroll_y: usize,
    width: usize,
    height: usize,
) -> Result<(), String> {
    queue!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .map_err(|e| format!("Failed to clear terminal: {}", e))?;

    let chapter = &chapters[current_chapter];

    let header_text = format!(
        " DOCKTURE INTERACTIVE MANUAL (Chapter {}/{}) ",
        current_chapter + 1,
        chapters.len()
    );
    let border_width = width.saturating_sub(header_text.len() + 4);
    let left_border = "═".repeat(border_width / 2);
    let right_border = "═".repeat(border_width - (border_width / 2));

    let header = format!(
        "╔{} {} {}╗",
        left_border,
        header_text.bold().green(),
        right_border
    );
    println!("{}", header);

    let mut rendered_lines = Vec::new();
    rendered_lines.push("".to_string());
    rendered_lines.push(format!("  {}", chapter.title.bold().cyan()));
    rendered_lines.push(
        "  ======================================================"
            .cyan()
            .to_string(),
    );
    rendered_lines.push("".to_string());

    for &line in chapter.content {
        rendered_lines.push(format!("  {}", line));
    }

    let viewport_height = height.saturating_sub(3);
    let start_idx = scroll_y;
    let end_idx = std::cmp::min(start_idx + viewport_height, rendered_lines.len());

    for i in start_idx..end_idx {
        let line = &rendered_lines[i];
        let mut truncated = if line.len() > width - 5 {
            format!("{}...", &line[0..width - 8])
        } else {
            line.clone()
        };

        if truncated.trim().starts_with('$') {
            truncated = truncated.yellow().to_string();
        } else if truncated.contains("•")
            || truncated.contains("-")
                && (truncated.contains("--")
                    || truncated.contains("install")
                    || truncated.contains("status"))
        {
            truncated = truncated.white().to_string();
        }

        let has_scroll = rendered_lines.len() > viewport_height;
        let is_scrollbar_area = i >= start_idx && i < end_idx;
        let scrollbar_char = if has_scroll && is_scrollbar_area {
            let scroll_ratio = start_idx as f64 / (rendered_lines.len() - viewport_height) as f64;

            let scroll_indicator_idx = (scroll_ratio * (viewport_height - 1) as f64) as usize;
            if i - start_idx == scroll_indicator_idx {
                "█".cyan().to_string()
            } else {
                "│".dark_grey().to_string()
            }
        } else {
            "║".to_string()
        };

        let plain_len = strip_ansi_length(&truncated);
        let right_padding_len = width.saturating_sub(plain_len + 4);
        let right_padding = " ".repeat(right_padding_len);
        println!("║ {}{} {}", truncated, right_padding, scrollbar_char);
    }

    let printed_count = end_idx - start_idx;
    if printed_count < viewport_height {
        for _ in 0..(viewport_height - printed_count) {
            println!("║{}║", " ".repeat(width - 2));
        }
    }

    let footer_border = "═".repeat(width - 2);
    println!("╚{}╝", footer_border);

    let footer_legend =
        " [Left/Right] Chapter  •  [Up/Down] Scroll  •  [Space/PgDn] Page  •  [Q/Esc] Exit"
            .to_string();
    let legend_len = strip_ansi_length(&footer_legend);
    let legend_padding = " ".repeat(width.saturating_sub(legend_len + 1));
    print!("{}{}", footer_legend.bold().white(), legend_padding);
    stdout
        .flush()
        .map_err(|e| format!("Failed to flush output: {}", e))?;

    Ok(())
}

fn strip_ansi_length(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() || c == 'm' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }

    len
}
