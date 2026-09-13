# architecture and design

dockture operates as a single static binary daemon connected directly to `/var/run/docker.sock` or a remote tcp/tls socket specified via `DOCKER_HOST`. when starting, it attaches an event reactor to the docker engine event stream to receive container lifecycle events (`die`, `oom`, `health_status`, `start`). incoming events pass through container matching rules (`monitored_containers` and `ignored_containers`). if a container is excluded by glob pattern, its events are dropped immediately.

when a container crash or error event is detected, dockture queries stdout and stderr buffers up to `log_tail_size` lines and appends the collected log snippet to alert payloads. separate log watcher tasks stream container output and scan lines against configured patterns (`log_keywords`, default: `error`, `fatal`, `fail`). alongside event monitoring, a background sampling loop runs every 30 seconds to fetch container resource usage from `/containers/{id}/stats?stream=false`. cpu percentage and memory usage are recorded into a rolling sample window of up to 60 samples, covering roughly 30 minutes, for statistical anomaly analysis.

when `auto_restart = true` and a container terminates with an exit error or oom kill, the self-healer issues a restart request. to limit restart feedback loops on broken containers, restart attempts are logged in a 5-minute (300 seconds) sliding window; once the restart threshold is reached, further self-healing restarts are suspended for that window and alerts continue with CrashLoopBackOff context. notification delivery currently runs inline with alert handling: smtp is attempted first, followed by discord and slack webhooks. if the docker connection or event stream drops, dockture stops the current monitor session, cancels session-scoped monitor tasks, reconnects with bounded exponential backoff, recreates the event subscription, and reconciles currently running containers so log watchers are recreated. docker events that occur while disconnected are not replayed by the current implementation.

| metric | typical value | operational detail |
|---|---|---|
| memory usage | ~1-2 mb rss | resident memory during active monitoring |
| cpu usage | < 0.1% | idle polling overhead on standard workloads |
| binary size | ~15 mb | static release executable compiled with rust |

---

previous: [home](./Home.md) | home: [home](./Home.md) | next: [statistical anomaly detection](./Statistical-Anomaly-Detection.md)
