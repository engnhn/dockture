# Statistical Anomaly Detection

Dockture uses Z-scores to detect resource usage spikes in CPU and memory metrics.

## Z-Score Calculation

The Z-score measures how many standard deviations a value is from the historical mean:

$$Z = \frac{X - \mu}{\sigma}$$

Here $X$ is the current CPU or RAM sample, $\mu$ is the rolling average, and $\sigma$ is the standard deviation. When the computed Z-score exceeds `anomaly_threshold` (default: 3.0), Dockture sends an anomaly warning alert.

To prevent division-by-zero errors when resource metrics remain flat, `anomaly_sensitivity` sets a minimum floor for the standard deviation denominator.

## State Persistence

Rolling metrics are saved to `~/.config/dockture/state_buffer.json` every 30 seconds with owner-only (`0600`) permissions.

On startup, Dockture checks the timestamp of `state_buffer.json`. If the file is older than 2 hours (7,200 seconds), stale state is discarded.
