# Notifications & Alert Routing

Dockture sends alerts via SMTP Email, Discord Webhooks, and Slack Webhooks.

## Alert Categories

Alerts are divided into four event categories:

| Category | Trigger Event |
|---|---|
| `crash` | Container process termination, exit error, or OOM kill |
| `warning` | Resource usage Z-score anomaly or log error match |
| `health` | Container health check status changed to unhealthy |
| `recovery` | Container restarted or health check returned to healthy |

Categories can be assigned to channels in `config.toml`:

```toml
email_alerts = ["crash", "health"]
discord_alerts = ["crash", "warning"]
slack_alerts = ["warning", "recovery"]
```

## Testing Notifications

Run test commands to verify channel settings before deploying the daemon:

```bash
dockture test-email
dockture test-webhook
```
