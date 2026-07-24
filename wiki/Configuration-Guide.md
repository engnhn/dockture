# Configuration and Security Guide

Managing application settings securely and intuitively is a critical operational requirement for container infrastructure tools. Dockture provides a robust configuration management system based on the TOML format, combining interactive CLI wizards, strict filesystem security policies, secret masking, and dynamic configuration updating tools. This architecture guarantees that sensitive credentials such as SMTP passwords and webhook secrets are stored safely on the host system while remaining easily manageable for operators.

By default, Dockture stores its configuration settings in the user's home directory at `~/.config/dockture/config.toml`. When the daemon or CLI commands execute, Dockture resolves the environment path dynamically, creating the directory hierarchy automatically if it does not yet exist. To protect confidential credentials from unauthorized inspection by other local system users or processes, Dockture enforces strict POSIX filesystem permissions upon creating or updating the file. Using Unix permission bits (`0o600`), read and write access is restricted exclusively to the file owner, preventing unprivileged system users from accessing sensitive SMTP passwords or internal webhook URLs.

```toml
# Example ~/.config/dockture/config.toml
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_user = "alerts@example.com"
smtp_pass = "super-secret-app-password"
sender_email = "dockture@example.com"
receiver_emails = ["devops@example.com", "oncall@example.com"]
log_tail_size = 100
auto_restart = true
anomaly_detection = true
anomaly_threshold = 3.0
anomaly_sensitivity = 0.2
ignored_containers = ["test-*", "temp-build-*"]
monitored_containers = ["prod-*", "db-*"]
discord_webhook = "https://discord.com/api/webhooks/123456789/abcdef..."
slack_webhook = "https://hooks.slack.com/services/T00/B00/X00"
email_alerts = ["crash", "health"]
discord_alerts = ["crash", "warning"]
slack_alerts = ["warning", "recovery"]
log_keywords = ["error", "fatal", "exception", "panic"]
```

Initial setup is simplified through the interactive wizard executed via `dockture init`. When invoked, the wizard prompts the operator step-by-step for key configuration items, including the SMTP host address, server port, authentication credentials, sender email address, target notification receivers, and log extraction tail sizes. Input masking is applied automatically during password prompts to prevent sensitive keys from appearing in terminal session scrollbacks. Once completed, the wizard validates the inputs, serializes the settings into clean TOML, and saves the configuration file with owner-only access permissions.

Operators can inspect and modify existing configuration settings at any time without manually opening the TOML file in a text editor. Executing `dockture config show` displays the active configuration in a formatted terminal table, automatically masking the SMTP password string for security. Individual configuration fields can be modified on the fly using the `dockture config set` subcommand with specific flags. For example, updating the log tail buffer size or toggling automatic container restarts can be accomplished seamlessly through single CLI invocations:

```bash
# Update log extraction tail size to 250 lines
dockture config set --log-tail-size 250

# Enable automated restart for crashed or unhealthy containers
dockture config set --auto-restart true

# Add a new receiver email address
dockture config add-receiver ops-team@example.com
```

To support flexible filtering across complex Docker environments, Dockture supports glob pattern matching for container inclusion and exclusion. Using the `monitored_containers` and `ignored_containers` configuration lists, operators can target specific container naming patterns (such as `prod-*` or `db-*`) while ignoring transient build containers or test suites (such as `test-*`). Furthermore, log keyword monitoring can be customized by defining custom matching terms in `log_keywords`, ensuring that Dockture scans log buffers for specialized error signatures unique to your stack.

---

## Custom Configuration Paths and Environment Overrides

While `~/.config/dockture/config.toml` serves as the default location, operational environments such as containerized deployments, system services, or CI/CD pipelines often require custom configuration paths. Dockture supports dynamic configuration path resolution via the `--config <PATH>` global CLI flag or the `DOCKTURE_CONFIG` environment variable.

```bash
# Execute daemon using a custom configuration file path
dockture --config /etc/dockture/production.toml run

# Alternatively, export DOCKTURE_CONFIG in containerized environments
export DOCKTURE_CONFIG=/etc/dockture/production.toml
dockture run
```

---

## Remote Docker Host Connections (`DOCKER_HOST`)

By default, Dockture establishes a connection to the local Docker UNIX socket (`/var/run/docker.sock`). In distributed environments or Docker-in-Docker setups, operators can point Dockture to remote Docker daemons by defining the `DOCKER_HOST` environment variable. Dockture automatically parses TCP, HTTPS, or custom UNIX socket URLs (`unix://`, `tcp://`, `https://`), enabling remote container monitoring without requiring local socket access.

```bash
# Monitor a remote Docker daemon over TCP
export DOCKER_HOST=tcp://192.168.1.100:2375
dockture status
```
