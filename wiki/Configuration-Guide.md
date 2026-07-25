# Configuration Guide

Dockture resolves its configuration file in the following order:

1. `--config <PATH>` global CLI flag
2. `DOCKTURE_CONFIG` environment variable
3. `~/.config/dockture/config.toml` (default path)

All configuration files are created with POSIX `0600` permissions (owner read/write only) to protect passwords and webhook URLs.

## Example Configuration

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
anomaly_sensitivity = 0.2

ignored_containers = ["test-*"]
monitored_containers = ["prod-*", "db-*"]

discord_webhook = "https://discord.com/api/webhooks/..."
slack_webhook = "https://hooks.slack.com/services/..."

email_alerts = ["crash", "health"]
discord_alerts = ["crash", "warning"]
slack_alerts = ["warning", "recovery"]

log_keywords = ["error", "fatal", "panic"]
```

## Configuration Commands

To create a new configuration interactively, run `dockture init`.

To view active configuration settings with masked passwords, run `dockture config show`.

To update specific configuration options from the CLI:

```bash
dockture config set --log-tail-size 250
dockture config set --auto-restart true
dockture config add-receiver ops-team@example.com
```

## Container Pattern Matching

Container monitoring can be restricted using glob patterns in `monitored_containers` (e.g. `prod-*`, `db-*`) and `ignored_containers` (e.g. `test-*`).

## Remote Docker Host

Set the `DOCKER_HOST` environment variable to connect to a remote Docker daemon over TCP or HTTPS:

```bash
export DOCKER_HOST=tcp://192.168.1.100:2375
dockture status
```
