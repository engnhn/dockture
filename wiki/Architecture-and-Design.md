# architecture and design

dockture operates as a single static binary daemon connected directly to `/var/run/docker.sock` or a remote tcp/tls socket specified via `DOCKER_HOST`. when starting, it attaches an event reactor to the docker engine event stream to receive container lifecycle events (`die`, `oom`, `health_status`, `start`, `destroy`). incoming events pass through container matching rules (`monitored_containers` and `ignored_containers`). if a container is excluded by glob pattern, its events are dropped immediately.

when a container crash or error event is detected, the log watcher queries stdout and stderr buffers up to `log_tail_size` lines (default: 100). it scans lines against configured patterns (`log_keywords`, default: `error`, `fatal`, `panic`) and appends matching log snippets directly to alert payloads. alongside event monitoring, a background sampling loop runs every 30 seconds to fetch container resource usage from `/containers/{id}/stats?stream=false`. cpu percentage and memory byte usage are recorded into a rolling sample window (up to 20 samples covering 10 minutes) for statistical anomaly analysis.

recovery and notification handling operate independently from event ingestion. when `auto_restart = true` and a container terminates with an exit error or oom kill, the self-healer issues a restart request. to prevent infinite restart loops on broken containers, restart attempts are logged in a 5-minute (300 seconds) sliding window; if more than 3 restarts occur within that window, further restarts are halted and a warning alert is sent. all notification delivery functions (smtp email, discord webhooks, slack webhooks) execute inside isolated `tokio::spawn` tasks so network timeouts or external HTTP latencies never block the primary event reactor. if the docker socket disconnects or the daemon restarts, dockture automatically reconnects using exponential backoff from 1 second up to 60 seconds.

| metric | typical value | operational detail |
|---|---|---|
| memory usage | ~1-2 mb rss | resident memory during active monitoring |
| cpu usage | < 0.1% | idle polling overhead on standard workloads |
| binary size | ~15 mb | static release executable compiled with rust |

---

previous: [home](./Home.md) | home: [home](./Home.md) | next: [statistical anomaly detection](./Statistical-Anomaly-Detection.md)
