# dockture

dockture is a docker container monitoring daemon written in rust. it connects to the local or remote docker daemon socket to track container lifecycle events, monitor resource usage metrics, analyze log output for errors, and deliver alert notifications across email, discord, and slack.

## index

| page | summary |
|---|---|
| [architecture and design](./Architecture-and-Design.md) | internal daemon structure, socket event processing, log tailing, resource monitoring, auto-restart limits, and async notification dispatching |
| [statistical anomaly detection](./Statistical-Anomaly-Detection.md) | z-score mathematical model, denominator clamping, metric sampling windows, state persistence, and buffer staleness purging |
| [configuration guide](./Configuration-Guide.md) | file resolution order, file permissions, complete config.toml layout, glob pattern matching rules, cli management flags, and remote host setup |
| [notification channels and alerting](./Notification-Channels-and-Alerting.md) | email, discord, and slack delivery formats, alert event categories (`crash`, `warning`, `health`, `recovery`), channel routing configuration, and testing commands |
| [daemon and systemd integration](./Daemon-and-Systemd-Integration.md) | systemd user unit setup, persistent linger mode configuration, service lifecycle commands, and docker compose deployment |
| [cli reference and operations](./CLI-Reference-and-Operations.md) | command line syntax, global options, subcommand reference table, flag options, and shell completion generation |

---

home: [home](./Home.md) | next: [architecture and design](./Architecture-and-Design.md)
