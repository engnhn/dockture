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

# log tailing & keyword filtering
log_tail_size = 100
log_keywords = ["error", "fatal", "panic", "exception"]
ignored_log_patterns = ["healthcheck", "isCritical"]

# container selection rules
ignored_containers = ["test-*", "staging-tmp-*"]
monitored_containers = ["prod-*", "db-*", "api-gateway"]

# self healing settings
auto_restart = true

# anomaly detection & noise prevention settings
anomaly_detection = true
anomaly_threshold = 3.0
anomaly_sensitivity = 2.0
anomaly_min_value_cpu = 25.0
anomaly_min_value_mem = 40.0
anomaly_cooldown_secs = 1800

# alert throttling & noise reduction
send_recovery_emails = false
alert_cooldown_secs = 900

# automated daily summary report
daily_report_enabled = true
daily_report_time = "08:00"

# webhook endpoints
discord_webhook = "https://discord.com/api/webhooks/123456789/abcdef..."
slack_webhook = "https://hooks.slack.com/services/T00000000/B00000000/XXXXX..."

# alert category routing
email_alerts = ["crash", "health", "daily_report"]
discord_alerts = ["crash", "warning"]
slack_alerts = ["warning", "recovery"]
```

container selection relies on glob pattern matching (`*`, `?`). patterns declared in `ignored_containers` take absolute precedence; any matching container is excluded from monitoring regardless of `monitored_containers`. if `monitored_containers` contains patterns, only containers matching at least one pattern are tracked. if `monitored_containers` is empty or omitted, all non-ignored containers are monitored.

dockture includes built-in alert noise reduction:
- `send_recovery_emails` (default: `false`): controls whether resource recovery notifications (e.g. CPU/Memory returning below 80%) dispatch emails.
- `alert_cooldown_secs` (default: `900`s / 15 minutes): rate-limits repeat log error keyword alerts and resource warnings per container.
- `anomaly_min_value_cpu` (default: `25.0`%) & `anomaly_min_value_mem` (default: `40.0`%): enforces absolute minimum resource usage floors before statistical z-score anomaly scoring triggers alerts.
- `daily_report_enabled` (default: `true`) & `daily_report_time` (default: `"08:00"`): schedules a 24-hour daily HTML summary email.
- `ignored_log_patterns`: array of substring patterns to skip during log error matching.

configuration options can be inspected or mutated via CLI commands such as `dockture init` (interactive setup), `dockture config show` (displays settings with masked passwords), `dockture config set` (updates specific keys), `dockture test-report` (triggers daily summary report test), and `dockture config add-receiver <email>` (appends a recipient). to target a remote docker daemon over tcp or tls, export the `DOCKER_HOST` environment variable before running dockture commands (`export DOCKER_HOST=tcp://192.168.1.100:2375`).

---

previous: [statistical anomaly detection](./Statistical-Anomaly-Detection.md) | home: [home](./Home.md) | next: [notification channels and alerting](./Notification-Channels-and-Alerting.md)
