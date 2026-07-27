<p align="center">
  <img src="./assets/logo.svg" alt="dockture logo" width="180" />
</p>

<p align="center">
  <a href="https://github.com/engnhn/dockture/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/engnhn/dockture/ci.yml?branch=master&style=flat-square&label=ci" alt="ci status"></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-mit-blue.svg?style=flat-square" alt="license"></a>
  <a href="https://github.com/engnhn/dockture/releases"><img src="https://img.shields.io/github/v/release/engnhn/dockture?style=flat-square&color=emerald" alt="release"></a>
</p>

# dockture

dockture monitors docker containers for crashes, resource spikes, and log errors. it sends notifications through email, discord, or slack.

## quick start

install the binary:

```bash
curl -fsSL https://raw.githubusercontent.com/engnhn/dockture/master/install.sh | sh
```

or build from source:

```bash
git clone https://github.com/engnhn/dockture.git
cd dockture
cargo build --release
sudo cp target/release/dockture /usr/local/bin/
```

## configuration

generate `~/.config/dockture/config.toml` or run `dockture init`:

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

run dockture in the foreground:

```bash
dockture run
```

run as a systemd user service:

```bash
dockture service install
dockture service start
```

## docker compose

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

## commands

| command | description |
|---|---|
| `dockture init` | interactive setup wizard |
| `dockture run` | start monitoring daemon |
| `dockture status` | display container metrics and disk usage |
| `dockture logs <container>` | stream colorized logs (`--tail`, `--follow`) |
| `dockture test-email` | test smtp configuration |
| `dockture test-webhook` | test discord and slack webhooks |
| `dockture config show` | print active configuration |
| `dockture config set [flags]` | update configuration options |
| `dockture service <action>` | manage systemd user service (`install`, `start`, `stop`, `status`) |
| `dockture manual` | open terminal manual |

## documentation

detailed documentation is in the `wiki/` directory.

## license

[mit](./LICENSE)
