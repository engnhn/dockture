# statistical anomaly detection

dockture calculates rolling z-scores to detect resource utilization spikes in cpu and memory before process failures occur. the z-score measures how many standard deviations a sample value deviates from the historical rolling mean:

$$z = \frac{x - \mu}{\max(\sigma, s)}$$

in this equation, $x$ represents the current cpu percentage or memory byte sample, $\mu$ is the rolling mean across the historical window, $\sigma$ is the sample standard deviation, and $s$ is the `anomaly_sensitivity` floor parameter (default: `0.2`). clamping the denominator with $s$ prevents division by zero and eliminates false positive alerts when container resource usage remains flat ($\sigma \to 0$).

resource statistics are sampled every 30 seconds, maintaining a rolling buffer of up to 20 samples (a 10-minute historical window). a minimum of 5 samples must be collected before anomaly scoring begins. if a calculated z-score exceeds `anomaly_threshold` (default: `3.0`), dockture flags the sample as a resource anomaly and dispatches a `warning` category notification.

rolling sample metrics are saved to `~/.config/dockture/state_buffer.json` every 30 seconds with POSIX `0600` owner-only file permissions to preserve historical baselines across daemon restarts. upon daemon startup, dockture inspects the file modification timestamp of `state_buffer.json`; if the file is older than 7,200 seconds (2 hours), the buffer is purged to prevent outdated historical statistics from generating inaccurate alerts after prolonged downtime.

---

previous: [architecture and design](./Architecture-and-Design.md) | home: [home](./Home.md) | next: [configuration guide](./Configuration-Guide.md)
