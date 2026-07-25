<p align="center">
  <img src="./assets/logo.svg" alt="Dockture Logo" width="180" />
</p>

<p align="center">
  <a href="https://github.com/engnhn/dockture/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/engnhn/dockture/ci.yml?branch=master&style=flat-square&label=CI" alt="CI Status"></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://github.com/engnhn/dockture/releases"><img src="https://img.shields.io/github/v/release/engnhn/dockture?style=flat-square&color=emerald" alt="Release"></a>
</p>

# Dockture

Dockture monitors Docker containers for crashes, resource spikes, and log errors, sending alerts via Email, Discord, or Slack.

## Quick Start

Install the pre-built binary:

```bash
curl -fsSL https://raw.githubusercontent.com/engnhn/dockture/master/install.sh | sh
```

Or build from source:

```bash
git clone https://github.com/engnhn/dockture.git
cd dockture
cargo build --release
sudo cp target/release/dockture /usr/local/bin/
```

## Configuration

Create `~/.config/dockture/config.toml` or run `dockture init` to use the interactive wizard:

```toml
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_user = "alerts@example.com"
smtp_pass = "app-password"
sender_email = "dockture@example.com"
receiver_emails = ["admin@example.com"]

auto_restart = true
anomaly_detection = true
anomaly_threshold = 3.0

discord_webhook = "https://discord.com/api/webhooks/..."
slack_webhook = "https://hooks.slack.com/services/..."

email_alerts = ["crash", "health"]
discord_alerts = ["crash", "warning"]
log_keywords = ["error", "fatal", "panic"]
```

Run Dockture in the foreground:

```bash
dockture run
```

To run Dockture as a systemd user service:

```bash
dockture service install
dockture service start
```

## Docker Compose

```yaml
version: "3.8"

services:
  dockture:
    image: engnhn/dockture:latest
    container_name: dockture
    restart: always
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ~/.config/dockture:/root/.config/dockture:ro
    environment:
      - DOCKTURE_CONFIG=/root/.config/dockture/config.toml
```

```bash
docker-compose up -d
```

## Commands

| Command | Description |
|---|---|
| `dockture init` | Interactive setup wizard |
| `dockture run` | Start monitoring daemon |
| `dockture status` | Show container metrics and disk usage |
| `dockture logs <container>` | Stream colorized logs (`--tail`, `--follow`) |
| `dockture test-email` | Test SMTP settings |
| `dockture test-webhook` | Test Discord and Slack webhooks |
| `dockture config show` | Print active configuration |
| `dockture config set [FLAGS]` | Update configuration keys |
| `dockture service <ACTION>` | Manage systemd service (`install`, `start`, `stop`, `status`) |
| `dockture manual` | Open terminal help guide |

## Documentation

Full documentation is available in the `wiki/` directory.

## License

[MIT](./LICENSE)
