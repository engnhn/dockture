# Architecture & Design

Dockture runs as a background daemon that connects directly to the local Docker socket (`/var/run/docker.sock`).

The event reactor listens for Docker daemon events such as container crashes (`die`), out-of-memory kills (`oom`), health status changes, and container starts.

When a crash or error event occurs, the log watcher tails stdout and stderr buffers to capture recent lines containing error keywords.

The resource analyzer samples CPU and memory usage every 30 seconds to compute rolling Z-scores. If a container crashes, the self-healer attempts to restart it, enforcing a limit of 3 restarts in 5 minutes to prevent infinite restart loops.

All notification requests are dispatched in separate background tasks so that network latencies or SMTP timeouts do not block event processing.

## Resource Usage

| Metric | Typical Value | Notes |
|---|---|---|
| Memory | ~1-2 MB RSS | Resident memory during active monitoring |
| CPU | < 0.1% | Idle CPU usage while listening to socket events |
| Binary Size | ~15 MB | Static release executable |

If the Docker daemon restarts, Dockture automatically attempts reconnection with exponential backoff until the connection is restored.
