<p align="center">
  <img src="./assets/logo.svg" alt="Dockture Logo" width="180" />
</p>

<p align="center">
  <a href="https://github.com/sampletheory/dockture/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/sampletheory/dockture/ci.yml?branch=main&style=flat-square&logo=github&label=CI" alt="CI Status"></a>
  <a href="https://crates.io"><img src="https://img.shields.io/badge/rust-2024_edition-orange.svg?style=flat-square&logo=rust" alt="Rust Edition"></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://github.com/sampletheory/dockture/releases"><img src="https://img.shields.io/github/v/release/sampletheory/dockture?style=flat-square&color=emerald" alt="Release"></a>
</p>

---

# Dockture - Docker Container Health & Anomaly Monitoring Daemon

Dockture is an event-driven container monitoring and automated recovery daemon written in Rust. It connects to the local UNIX Docker socket (`/var/run/docker.sock`) or remote TCP/HTTPS endpoints to process real-time Docker events (`die`, `oom`, `health_status`), calculate statistical Z-score anomalies for CPU and memory metrics, scan container log buffers for fatal signatures, and send diagnostic notifications via SMTP Email, Discord Webhooks, and Slack Webhooks.

---

## Technical Architecture

```
                  +-------------------------------------------------+
                  |                Docker Daemon                    |
                  |          (/var/run/docker.sock)                 |
                  +------------------------+------------------------+
                                           |
                                  Event & Stats Stream
                                           |
                                           v
                  +-------------------------------------------------+
                  |                Dockture Daemon                  |
                  |                                                 |
                  |  +-------------------+   +-------------------+  |
                  |  |  Event Reactor    |   |  Z-Score Engine   |  |
                  |  +---------+---------+   +---------+---------+  |
                  |            |                       |            |
                  |            +-----------+-----------+            |
                  |                        |                        |
                  |                        v                        |
                  |             +---------------------+             |
                  |             | Multi-Channel Router|             |
                  |             +----------+----------+             |
                  +------------------------|------------------------+
                                           |
                    +----------------------+----------------------+
                    |                      |                      |
                    v                      v                      v
            +---------------+      +---------------+      +---------------+
            |  SMTP Email   |      |    Discord    |      |     Slack     |
            +---------------+      +---------------+      +---------------+
```

Dockture executes on the Tokio asynchronous runtime using the `bollard` client library. The architecture comprises five core components:

1. **Event Reactor (`src/monitor/event_reactor.rs`)**: Listens to non-blocking Docker event streams and reacts to state changes (`die`, `oom`, `health_status`, `start`).
2. **Log Watcher (`src/monitor/log_watcher.rs`)**: Streams container stdout and stderr buffers to detect configurable error keywords (`error`, `fatal`, `exception`, `panic`).
3. **Resource Analyzer (`src/monitor/resource_analyzer.rs`)**: Queries CPU and RAM statistics every 30 seconds, tracks host disk usage, and evaluates rolling Z-scores ($Z = \frac{x - \mu}{\sigma}$).
4. **Self Healer (`src/monitor/self_healer.rs`)**: Restarts failed containers and enforces a 5-minute sliding window CrashLoopBackOff threshold ($\ge 3$ restarts in 300 seconds).
5. **Notification Router (`src/notifier/mod.rs`)**: Dispatches HTML and plain-text email alerts or formatted webhook payloads based on event categories (`crash`, `health`, `warning`, `recovery`).

---

## Technical Specifications

| Parameter | Specification |
|---|---|
| Runtime | Rust (Tokio Asynchronous Event Loop) |
| Docker API Client | Bollard 0.15 |
| Configuration Storage | TOML Format (`~/.config/dockture/config.toml`) |
| Filesystem Security | POSIX `0600` Permissions (Owner Read/Write Only) |
| Container Connection | Local Socket (`/var/run/docker.sock`) or `DOCKER_HOST` (`tcp://`, `https://`, `unix://`) |
| Binary Size | ~15 MB (Multi-stage Alpine Build) |

---

## Configuration Reference

The configuration file is resolved in the following precedence:
1. `--config <PATH>` global CLI flag
2. `DOCKTURE_CONFIG` environment variable
3. Default path: `~/.config/dockture/config.toml`

### Example Configuration (`config.toml`)

```toml
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_user = "alerts@example.com"
smtp_pass = "app-specific-password"
sender_email = "dockture@example.com"
receiver_emails = ["admin1@example.com", "admin2@example.com"]
log_tail_size = 100
auto_restart = true
anomaly_detection = true
anomaly_threshold = 3.0
anomaly_sensitivity = 0.2
ignored_containers = ["test-*", "temp-*"]
monitored_containers = ["prod-*", "db-*"]
discord_webhook = "https://discord.com/api/webhooks/123456789/abcdef..."
slack_webhook = "https://hooks.slack.com/services/T00/B00/X00"
email_alerts = ["crash", "health"]
discord_alerts = ["crash", "warning"]
slack_alerts = ["warning", "recovery"]
log_keywords = ["error", "fatal", "exception", "panic"]
```

---

## Installation & Deployment

### 1. Quick One-Line Script Installation (Linux)

Install the latest pre-compiled binary automatically for your architecture (`x86_64` or `aarch64`):

```bash
curl -fsSL https://raw.githubusercontent.com/sampletheory/dockture/master/install.sh | sh
```

---

### 2. Docker & Docker Compose

Deploying via Docker requires mounting the host Docker socket and configuration directory as read-only volumes:

```yaml
version: "3.8"

services:
  dockture:
    image: sampletheory/dockture:latest
    container_name: dockture-daemon
    restart: always
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ~/.config/dockture:/root/.config/dockture:ro
    environment:
      - DOCKTURE_CONFIG=/root/.config/dockture/config.toml
```

Run using Docker Compose:
```bash
docker-compose up -d
```

### 2. Systemd User Service (Linux)

Install Dockture as an unprivileged systemd user service:

```bash
# Install systemd unit file to ~/.config/systemd/user/dockture.service
dockture service install

# Enable and start the service
dockture service start

# Query service status
dockture service status
```

Enable user lingering so the daemon starts automatically at system boot without an active SSH session:
```bash
loginctl enable-linger $USER
```

### 3. Cargo Build

```bash
git clone https://github.com/sampletheory/dockture.git
cd dockture
cargo build --release
sudo cp target/release/dockture /usr/local/bin/
```

---

## CLI Command Reference

### Global Options
- `--config <PATH>`: Specify custom configuration file path (or set `DOCKTURE_CONFIG`).

### Commands

| Command | Description |
|---|---|
| `dockture init` | Launches interactive wizard to create configuration file. |
| `dockture run` | Launches the monitoring daemon in the foreground. |
| `dockture status` | Displays container ID, name, image, state, status, and host disk usage. |
| `dockture logs <CONTAINER>` | Streams colorized log output with keyword highlighting (`--tail`, `--follow`). |
| `dockture test-email` | Sends diagnostic test email via configured SMTP server. |
| `dockture test-webhook` | Sends diagnostic test payloads to configured Discord and Slack webhooks. |
| `dockture config show` | Displays active settings with masked SMTP password. |
| `dockture config set [FLAGS]` | Modifies specific configuration keys. |
| `dockture config add-receiver <EMAIL>` | Appends recipient email address. |
| `dockture config remove-receiver <EMAIL>` | Removes recipient email address. |
| `dockture service <SUBCOMMAND>` | Manages systemd user service (`install`, `uninstall`, `start`, `stop`, `restart`, `status`). |
| `dockture manual` | Launches interactive terminal user manual. |
| `dockture complete <SHELL>` | Generates shell completion script (`bash`, `zsh`, `fish`). |

---

## Wiki Documentation Index

Detailed architectural specs, formulas, and operational guides are documented in the [/wiki](./wiki/) directory:

- [Home](./wiki/Home.md) - Main documentation portal overview.
- [Architecture & Design](./wiki/Architecture-and-Design.md) - Event reactor, concurrency model, and threat boundaries.
- [Statistical Anomaly Detection](./wiki/Statistical-Anomaly-Detection.md) - Z-score formulas, standard deviation clamps, and cooldown logic.
- [Configuration Guide](./wiki/Configuration-Guide.md) - TOML schema, POSIX permissions, and `DOCKER_HOST` remote sockets.
- [Notification Channels & Alerting](./wiki/Notification-Channels-and-Alerting.md) - SMTP, Discord Embeds, Slack Block Kit, and alert routing.
- [Daemon & Systemd Integration](./wiki/Daemon-and-Systemd-Integration.md) - Systemd unit configurations and persistent linger setup.
- [CLI Reference & Operations](./wiki/CLI-Reference-and-Operations.md) - Detailed argument specifications for all subcommands.

---

## License

This project is licensed under the MIT License. See [LICENSE](./LICENSE) for details.
Copyright (c) 2026 sampletheory.
