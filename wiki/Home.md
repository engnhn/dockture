# Welcome to the Dockture Documentation Wiki

The official documentation wiki for **Dockture** provides in-depth technical guides, architectural breakdowns, operational manuals, and mathematical specifications covering every aspect of the project. Designed for system administrators, DevOps engineers, and Rust developers alike, this suite of documentation serves as a comprehensive resource for understanding how Dockture monitors, analyzes, and protects containerized environments.

Dockture is a high-performance, zero-dependency container physician written in Rust. It functions as an automated real-time daemon that streams lifecycle events directly from the local UNIX Docker socket, calculates statistical Z-score anomalies across CPU and RAM usage, scans log buffers for critical keywords, and dispatches detailed multi-channel alerts via SMTP Email, Discord, and Slack.

---

## Documentation Navigation

### 1. [Architecture and Design](./Architecture-and-Design.md)
Explore the internal engineering of Dockture, including the non-blocking Tokio event loop, asynchronous Docker socket integration via Bollard, low-memory footprint strategies, and release compilation optimizations.

### 2. [Statistical Anomaly Detection](./Statistical-Anomaly-Detection.md)
Read a detailed breakdown of the statistical algorithm powering Dockture's anomaly detection engine. Learn how rolling standard deviations and adaptive Z-scores are computed to identify erratic resource consumption spikes.

### 3. [Configuration and Security Guide](./Configuration-Guide.md)
Discover how Dockture handles configuration file parsing, strict POSIX permissions (`0600`), secret masking, interactive setup wizards, and fine-grained option modifications using the CLI.

### 4. [Notification Channels and Alert Routing](./Notification-Channels-and-Alerting.md)
Understand how Dockture formats and dispatches alerts across SMTP Email, Discord Webhooks, and Slack Webhooks. Learn how to configure custom category routing rules to direct critical crashes and resource warnings to specific channels.

### 5. [Daemon and Systemd Integration](./Daemon-and-Systemd-Integration.md)
Learn how to deploy Dockture as a continuous background monitoring service on Linux systems using native systemd user service commands (`dockture service`).

### 6. [CLI Reference and Operations](./CLI-Reference-and-Operations.md)
Access the complete command-line interface manual detailing every subcommand, flag, argument, and interactive terminal interface supported by Dockture.

---

## Community and Support

Dockture is an open-source project maintained under the `sampletheory` organization. If you encounter bugs, require additional features, or wish to contribute improvements to the codebase, please visit our official GitHub repository at [github.com/sampletheory/dockture](https://github.com/sampletheory/dockture).
