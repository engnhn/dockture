# Notification Channels and Alert Routing

Delivering clear, actionable diagnostic information during production incidents is essential for rapid incident response. When a container failure, health check degradation, or statistical resource anomaly occurs, engineers need immediate access to contextual details such as container names, exit codes, failure classifications, and relevant log lines. Dockture incorporates a flexible, multi-channel notification engine capable of formatting and dispatching rich diagnostic alerts across SMTP Email, Discord Webhooks, and Slack Webhooks.

The SMTP notification driver is powered by the `lettre` library, supporting secure TLS transport (`STARTTLS` and implicit TLS over port 465) and authenticated message submission. When sending email notifications, Dockture utilizes pre-compiled HTML templates engineered to render cleanly across modern email clients and mobile devices. Email alerts contain formatted status headers, container identifiers, timestamp records, exit status details, and a styled log code block displaying the tail end of the container's standard output and standard error logs leading up to the failure. This ensures that engineers receiving email alerts on mobile devices can quickly diagnose the root cause without opening a terminal session.

```
+-----------------------------------------------------------------------------------+
|                            Multi-Channel Alert Dispatch                           |
|                                                                                   |
|                               +------------------+                                |
|                               | Dockture Event   |                                |
|                               +--------+---------+                                |
|                                        |                                          |
|                                        v                                          |
|                               +------------------+                                |
|                               | Category Router  |                                |
|                               +--------+---------+                                |
|                                        |                                          |
|                 +----------------------+----------------------+                   |
|                 |                      |                      |                   |
|                 v                      v                      v                   |
|       Category: "crash"       Category: "warning"    Category: "recovery"         |
|                 |                      |                      |                   |
|                 v                      v                      v                   |
|         +---------------+      +---------------+      +---------------+           |
|         |  SMTP Driver  |      | Discord Driver|      |  Slack Driver |           |
|         +---------------+      +---------------+      +---------------+           |
|                 |                      |                      |                   |
+-----------------|----------------------|----------------------|-------------------+
                  v                      v                      v
          Formatted HTML            Discord Embed          Slack Attachment
             Email                     Webhook                 Payload
```

For teams relying on chatops and real-time collaboration platforms, Dockture provides native integration with Discord and Slack webhooks. Rather than sending plain unformatted text strings, the notification engine constructs platform-native payload structures using rich embed cards in Discord and formatted block attachments in Slack. Color-coded side borders visually demarcate event severities—red for critical crashes and OOM terminations, amber for resource anomalies or unhealthy states, and green for container recovery events.

Granular alert routing represents another key capability of Dockture's notification framework. Operational environments often generate different tiers of alerts that require routing to distinct channels. For instance, critical application crashes may require high-priority email notifications sent to an on-call rotation, whereas minor CPU resource warnings should be directed to a casual Discord channel. Dockture enables this flexibility through category routing configuration options (`email_alerts`, `discord_alerts`, `slack_alerts`). Operators can assign specific alert event categories—such as `crash`, `warning`, `health`, and `recovery`—to individual channels, creating a tailored alerting matrix.

To verify that notification settings and network routes are configured correctly prior to placing the daemon into production, Dockture includes a dedicated verification subcommand. Running `dockture test-email` constructs a mock diagnostic alert payload and dispatches it through the configured SMTP server to all receiver addresses. If transmission fails due to incorrect authentication credentials, firewall blocks, or invalid hostnames, the CLI prints an explicit diagnostic error trace, allowing operators to troubleshoot network and authentication parameters immediately.
