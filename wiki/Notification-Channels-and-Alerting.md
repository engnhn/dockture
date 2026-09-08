# notification channels and alert routing

dockture supports notification delivery across three channels: smtp email (using STARTTLS/TLS encryption to send structured HTML messages to addresses in `receiver_emails`), discord webhooks (posting HTTP payloads with color-coded rich embeds), and slack webhooks (posting HTTP payloads formatted with slack block kit section fields and monospace code blocks for log output).

alert events are classified into five distinct categories:

| category | trigger event | default level |
|---|---|---|
| `crash` | process termination, exit code failure, OOM kernel kill, or container crash | critical |
| `warning` | cpu or memory usage z-score anomaly, or log error keyword match | warning |
| `health` | container healthcheck status transition to `unhealthy` | warning |
| `recovery` | container restarted by self-healer, or healthcheck status returned to `healthy` | info |
| `daily_report` | automated 24-hour daily health summary email report | info |

log error keyword alerts feature high-precision word boundary and JSON value parsing (`is_valid_keyword_match`). matches within composite identifiers (e.g. `isCritical`, `errorCount`) or boolean/null value declarations (e.g. `"isCritical": false`, `"error": null`, `"failed": false`) are automatically filtered and suppressed.

channel delivery is configured by declaring category lists in `config.toml` (`email_alerts`, `discord_alerts`, `slack_alerts`). if a category list is omitted or left empty, all categories are delivered to that channel. notification settings can be verified prior to deployment using CLI test commands: `dockture test-email` sends a test email, `dockture test-report` generates a test 24-hour summary report email, and `dockture test-webhook` sends synthetic test payloads to configured discord and slack webhook URLs.

---

previous: [configuration guide](./Configuration-Guide.md) | home: [home](./Home.md) | next: [daemon and systemd integration](./Daemon-and-Systemd-Integration.md)
