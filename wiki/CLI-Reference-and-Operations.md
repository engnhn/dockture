# cli reference

dockture commands follow the standard execution format `dockture [global_options] <subcommand> [flags]`. the global flag `--config <path>` (or `-c <path>`) overrides the default configuration file path `~/.config/dockture/config.toml`.

| subcommand | arguments & flags | description |
|---|---|---|
| `dockture init` | none | interactive setup wizard creating `~/.config/dockture/config.toml` with `0600` permissions |
| `dockture run` | none | starts monitoring daemon in the foreground |
| `dockture status` | none | displays terminal status table with active containers, state, cpu %, and memory usage |
| `dockture logs <container>` | `--tail <lines>`, `--follow` | streams colorized container logs (`--tail` defaults to 100 lines, `--follow` streams continuously) |
| `dockture test-email` | none | sends a test email to `receiver_emails` via configured smtp server |
| `dockture test-webhook` | none | posts test payloads to `discord_webhook` and `slack_webhook` endpoints |
| `dockture config show` | none | prints active configuration with sensitive values (`smtp_pass`) masked |
| `dockture config set` | `--log-tail-size <int>`, `--auto-restart <bool>`, `--anomaly-detection <bool>`, `--anomaly-threshold <float>`, `--anomaly-sensitivity <float>` | updates configuration parameters directly from the command line |
| `dockture config add-receiver` | `<email>` | appends a new recipient email address to `receiver_emails` |
| `dockture service` | `<action>` (`install`, `start`, `stop`, `restart`, `status`, `uninstall`) | manages background systemd user service unit |
| `dockture manual` | none | opens terminal manual interface |
| `dockture complete` | `<shell>` (`bash`, `zsh`, `fish`, `powershell`) | generates shell completion script for stdout redirect |

shell completion scripts can be generated and installed using standard shell paths:

```bash
# bash
dockture complete bash > ~/.local/share/bash-completion/completions/dockture

# zsh
dockture complete zsh > ~/.zsh/completion/_dockture

# fish
dockture complete fish > ~/.config/fish/completions/dockture.fish
```

---

previous: [daemon and systemd integration](./Daemon-and-Systemd-Integration.md) | home: [home](./Home.md)
