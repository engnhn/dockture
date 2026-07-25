# Dockture Documentation

Dockture monitors Docker containers for crashes, resource anomalies, and log errors, sending alerts to Email, Discord, or Slack.

| Guide | Content |
|---|---|
| [Architecture](./Architecture-and-Design.md) | Event reactor, log tailing, metrics collection, and memory footprint |
| [Anomaly Detection](./Statistical-Anomaly-Detection.md) | Z-score calculations, sensitivity clamping, and state persistence |
| [Configuration](./Configuration-Guide.md) | TOML configuration format, POSIX file permissions, and `DOCKER_HOST` options |
| [Notifications](./Notification-Channels-and-Alerting.md) | SMTP email settings, Discord/Slack webhooks, and category routing |
| [Daemon & Systemd](./Daemon-and-Systemd-Integration.md) | Systemd user service setup, persistent linger mode, and Docker Compose |
| [CLI Reference](./CLI-Reference-and-Operations.md) | Commands, flags, arguments, and shell completion generation |
