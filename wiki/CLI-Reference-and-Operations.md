# Complete CLI Reference and Operations Manual

Dockture features a versatile command-line interface powered by `clap` v4, designed to serve both as a continuous background daemon and as an interactive operational utility. This reference manual provides a detailed breakdown of all available subcommands, flags, arguments, and operational workflows supported by Dockture.

The command-line syntax follows standard subcommand patterns:

```bash
dockture [GLOBAL_OPTIONS] [SUBCOMMAND] [FLAGS] [OPTIONS]
```

## Global Options

- `--config <PATH>`: Custom path to the `config.toml` file. Can also be set globally via the `DOCKTURE_CONFIG` environment variable. (Default: `~/.config/dockture/config.toml`).

---

## Subcommands Overview

### 1. `dockture init`
Launches an interactive setup wizard that guides the operator through configuring SMTP server settings, sender credentials, receiver email addresses, log tail limits, and notification options. Upon completion, settings are validated and written to `~/.config/dockture/config.toml` with POSIX `0600` permissions.

```bash
# Launch interactive configuration wizard
dockture init
```

---

### 2. `dockture run`
Launches the core Dockture monitoring daemon in the foreground. Once active, the daemon connects asynchronously to the local Docker socket (`/var/run/docker.sock`), streaming container events, calculating statistical Z-score anomalies, scanning log outputs, and dispatching multi-channel alerts in real time.

```bash
# Run the daemon in foreground execution mode
dockture run
```

---

### 3. `dockture status`
Fetches real-time status metrics for all active containers from the Docker API and renders them inside a clean, formatted terminal dashboard table. The output details container names, image tags, current execution state, health status indicators, CPU usage percentages, and RAM consumption figures.

```bash
# Render interactive container status dashboard
dockture status
```

---

### 4. `dockture logs`
Streams real-time colorized logs from a specified target container directly to the terminal interface. Includes automatic keyword colorization, highlighting critical patterns such as `ERROR`, `FATAL`, `EXCEPTION`, and `PANIC` to streamline troubleshooting.

**Arguments:**
- `<CONTAINER>`: The name or container ID to stream logs from (Required).
- `--tail <LINES>`: Number of trailing log lines to fetch from the log buffer (Default: `100`).
- `--follow`: Stream new log output continuously in real time.

```bash
# View last 50 lines of logs for a web container
dockture logs web-api --tail 50

# Follow live log stream continuously
dockture logs database-server --follow
```

---

### 5. `dockture test-email`
Constructs a mock diagnostic alert payload containing simulated container crash metadata and attempts to send it through the configured SMTP server to all registered receiver email addresses. Used to verify SMTP host connectivity, port settings, and authentication credentials.

```bash
# Send a test diagnostic email alert
dockture test-email
```

---

### 6. `dockture test-webhook`
Constructs mock alert payloads and attempts to transmit them to configured Discord and Slack webhook URLs. Used to verify webhook URL endpoints, network routes, and channel permissions prior to daemon deployment.

```bash
# Send test diagnostic payloads to Discord and Slack webhooks
dockture test-webhook
```

---

### 6. `dockture config`
Provides fine-grained subcommands for inspecting and updating configuration parameters without needing to manually edit the underlying TOML file.

**Subcommands:**
- `show`: Displays current settings in a terminal table with masked SMTP passwords.
- `add-receiver <EMAIL>`: Adds a new recipient email address to the receiver list.
- `remove-receiver <EMAIL>`: Removes an existing recipient email address.
- `set [FLAGS]`: Updates specific configuration key-value options.

**Available Flags for `set`:**
- `--smtp-host <HOST>`: Update SMTP server hostname.
- `--smtp-port <PORT>`: Update SMTP port (e.g. 587 or 465).
- `--smtp-user <USER>`: Update SMTP authentication username.
- `--smtp-pass <PASS>`: Update SMTP authentication password.
- `--sender-email <EMAIL>`: Update sender email address.
- `--log-tail-size <SIZE>`: Update log extraction tail line limit.
- `--discord-webhook <URL>`: Set Discord webhook URL (Use empty string `""` to clear).
- `--slack-webhook <URL>`: Set Slack webhook URL (Use empty string `""` to clear).
- `--ignored-containers <PATTERNS>`: Comma-separated glob patterns to ignore (e.g. `"test-*,temp-*"`).
- `--monitored-containers <PATTERNS>`: Comma-separated glob patterns to monitor (e.g. `"prod-*,db-*"`).
- `--email-alerts <CATEGORIES>`: Comma-separated alert categories for email (e.g. `"crash,health"`).
- `--discord-alerts <CATEGORIES>`: Comma-separated alert categories for Discord (e.g. `"crash,warning"`).
- `--slack-alerts <CATEGORIES>`: Comma-separated alert categories for Slack (e.g. `"warning,recovery"`).
- `--auto-restart <BOOL>`: Enable or disable automated container restarts (`true`/`false`).
- `--log-keywords <KEYWORDS>`: Comma-separated log search terms (e.g. `"error,fatal,panic"`).
- `--anomaly-detection <BOOL>`: Enable or disable statistical Z-score anomaly tracking.
- `--anomaly-threshold <FLOAT>`: Set Z-score threshold multiplier (e.g. `3.0`).
- `--anomaly-sensitivity <FLOAT>`: Set standard deviation variance floor sensitivity (e.g. `0.2`).

```bash
# Show configuration summary
dockture config show

# Add a recipient email address
dockture config add-receiver ops@example.com

# Update Z-score anomaly detection threshold
dockture config set --anomaly-threshold 3.5
```

---

### 7. `dockture service`
Manages the lifecycle of the systemd user service unit for Dockture on Linux host environments.

**Subcommands:**
- `install`: Creates and registers `~/.config/systemd/user/dockture.service`.
- `uninstall`: Stops, disables, and removes the systemd user service unit file.
- `start`: Starts the background systemd user service.
- `stop`: Stops the running systemd user service.
- `restart`: Restarts the systemd user service.
- `status`: Displays current systemd service execution state.

```bash
# Install and start the background service
dockture service install
dockture service start

# Query service status
dockture service status
```

---

### 8. `dockture manual`
Launches an interactive terminal-based user manual powered by `dialoguer`. Allows operators to navigate through embedded topics, architectural notes, configuration guides, and troubleshooting steps directly inside the terminal interface without needing internet access.

```bash
# Launch interactive terminal manual
dockture manual
```

---

### 9. `dockture complete`
Generates shell autocompletion scripts for popular command-line shells, enabling tab-completion for all Dockture subcommands, options, and flags.

**Supported Shells:** `bash`, `zsh`, `fish`, `powershell`, `elvish`.

```bash
# Generate autocompletion for Bash
dockture complete bash > ~/.local/share/bash-completion/completions/dockture

# Generate autocompletion for Zsh
dockture complete zsh > ~/.zsh/completion/_dockture
```
