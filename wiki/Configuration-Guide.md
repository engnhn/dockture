# configuration guide

dockture resolves its configuration file by checking three sources in order: the explicit `--config <path>` CLI flag, the `DOCKTURE_CONFIG` environment variable, and the default file path `~/.config/dockture/config.toml`. all configuration files created or modified by dockture are written with POSIX `0600` file permissions (owner read and write only) to protect plain-text smtp passwords and webhook tokens.

```toml
# smtp server settings
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_user = "alerts@example.com"
smtp_pass = "app-password"
sender_email = "dockture@example.com"
receiver_emails = ["admin@example.com", "ops@example.com"]

# log tailing settings
log_tail_size = 100
log_keywords = ["error", "fatal", "panic", "exception"]

# container selection rules
ignored_containers = ["test-*", "staging-tmp-*"]
monitored_containers = ["prod-*", "db-*", "api-gateway"]

# self healing settings
auto_restart = true

# anomaly detection settings
anomaly_detection = true
anomaly_threshold = 3.0
anomaly_sensitivity = 0.2

# webhook endpoints
discord_webhook = "https://discord.com/api/webhooks/123456789/abcdef..."
slack_webhook = "https://hooks.slack.com/services/T00000000/B00000000/XXXXX..."

# alert category routing
email_alerts = ["crash", "health"]
discord_alerts = ["crash", "warning"]
slack_alerts = ["warning", "recovery"]
```

container selection relies on glob pattern matching (`*`, `?`). patterns declared in `ignored_containers` take absolute precedence; any matching container is excluded from monitoring regardless of `monitored_containers`. if `monitored_containers` contains patterns, only containers matching at least one pattern are tracked. if `monitored_containers` is empty or omitted, all non-ignored containers are monitored. configuration options can be inspected or mutated via CLI commands such as `dockture init` (interactive setup), `dockture config show` (displays settings with masked passwords), `dockture config set` (updates specific keys), and `dockture config add-receiver <email>` (appends a recipient). to target a remote docker daemon over tcp or tls, export the `DOCKER_HOST` environment variable before running dockture commands (`export DOCKER_HOST=tcp://192.168.1.100:2375`).

---

previous: [statistical anomaly detection](./Statistical-Anomaly-Detection.md) | home: [home](./Home.md) | next: [notification channels and alerting](./Notification-Channels-and-Alerting.md)
