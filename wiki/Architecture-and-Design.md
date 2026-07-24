# Architecture and Design Philosophy

The architecture of Dockture is built upon the principles of ultra-low overhead, non-blocking asynchronous execution, and high reliability under heavy system loads. In containerized production environments, a monitoring utility must never become a burden to the host operating system. Traditional monitoring stacks often consume significant memory and CPU cycles simply to poll metrics, which can degrade application performance on resource-constrained servers or edge nodes. Dockture addresses this fundamental problem by leveraging Rust's safety guarantees and zero-cost abstractions to deliver a lightweight, event-driven daemon that operates continuously with minimal footprint.

At the core of Dockture lies an event reactor loop constructed on the Tokio asynchronous runtime. Rather than executing periodic HTTP requests or spawning external subprocesses to check container health, Dockture establishes a persistent connection directly to the local UNIX Docker socket (`/var/run/docker.sock`) using the `bollard` library. Through this socket, the Docker engine streams native JSON event messages directly to Dockture whenever a container state change occurs. The daemon receives events such as container crashes (`die`), out-of-memory terminations (`oom`), health status changes (`health_status`), and lifecycle state transitions (`start`, `stop`, `destroy`) in real time, processing each payload instantaneously without blocking the execution thread.

```
+-----------------------------------------------------------------------------------+
|                                 Host System                                       |
|                                                                                   |
|  +-----------------------+              UNIX Socket Stream                        |
|  |     Docker Engine     | ===========================================+           |
|  +-----------------------+                                            |           |
|                                                                       v           |
|  +-----------------------------------------------------------------------------+  |
|  |                              Dockture Daemon                                |  |
|  |                                                                             |  |
|  |   +---------------------+   +---------------------+   +------------------+  |  |
|  |   | Tokio Event Reactor |   | Statistical Engine  |   |  Log Tail Buffer |  |  |
|  |   +----------+----------+   +----------+----------+   +--------+---------+  |  |
|  |              |                         |                       |            |  |
|  |              +-------------------------+-----------------------+            |  |
|  |                                        |                                    |  |
|  |                                        v                                    |  |
|  |                             +--------------------+                          |  |
|  |                             | Notification Dispatch|                        |  |
|  |                             +----------+---------+                          |  |
|  +----------------------------------------|------------------------------------+  |
|                                           |                                       |
+-------------------------------------------|---------------------------------------+
                                            v
                             SMTP / Discord / Slack Outbound
```

Concurrency in Dockture is managed through Tokio's task spawning model, allowing independent asynchronous tasks to handle event streaming, metrics processing, log extraction, and network notifications concurrently. When a critical container failure is detected, the event reactor spawns a dedicated background task to extract the recent log tail from the Docker API and dispatch formatted notifications. This ensures that slow network connections or SMTP server timeouts during alert delivery never delay the processing of subsequent Docker events or stall the main reactor loop.

Memory efficiency and binary compilation optimizations have been carefully engineered into the project's build profile. In `Cargo.toml`, Dockture configures release builds with maximum Link-Time Optimization (`lto = true`), size-focused optimization levels (`opt-level = "z"`), single code-generation units, and stripped debug symbols. Furthermore, panic behavior is set to abort immediately rather than unwinding call stacks, reducing runtime footprint and eliminating unnecessary metadata overhead. As a result, the compiled Dockture binary typically compiles to a compact executable that runs seamlessly across server nodes and embedded ARM platforms alike.

Configuration and state management within the daemon are designed to be thread-safe and resilient against runtime crashes. Application settings are loaded into an immutable thread-shared structure wrapped in atomic references, allowing all monitoring tasks to read configuration state safely without locking bottlenecks. If the local Docker daemon restarts or temporarily drops socket connections, Dockture gracefully attempts exponential backoff reconnection strategies until socket communication is re-established, guaranteeing unattended long-term operational resilience.
