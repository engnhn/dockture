# CLI Reference

Syntax:

```bash
dockture [GLOBAL_OPTIONS] [SUBCOMMAND] [FLAGS]
```

## Global Options

The global flag `--config <PATH>` sets a custom configuration file path (default: `~/.config/dockture/config.toml`).

## Subcommands

| Command | Description |
|---|---|
| `dockture init` | Interactive configuration setup wizard |
| `dockture run` | Run monitoring daemon in foreground |
| `dockture status` | Render container status and resource usage table |
| `dockture logs <container>` | Stream container logs (`--tail <lines>`, `--follow`) |
| `dockture test-email` | Send test SMTP alert |
| `dockture test-webhook` | Send test webhooks to Discord and Slack |
| `dockture config show` | Display active configuration settings |
| `dockture config set [FLAGS]` | Update specific configuration values |
| `dockture service <ACTION>` | Manage systemd service (`install`, `uninstall`, `start`, `stop`, `restart`, `status`) |
| `dockture manual` | Interactive terminal help manual |
| `dockture complete <shell>` | Generate shell completions (`bash`, `zsh`, `fish`, `powershell`) |
