# Statistical Anomaly Detection Engine

Detecting application failures before total system outages occur requires monitoring mechanisms that extend beyond simple binary health checks or static threshold alerts. While static threshold alerts (such as alerting when CPU usage exceeds 90%) can be useful, they frequently produce false positive alerts during expected peak workloads or fail to detect insidious memory leaks that gradually consume container memory over several hours. To solve this problem, Dockture integrates a dedicated statistical anomaly detection engine based on dynamic Z-score calculations and adaptive standard deviation tracking.

The Z-score (also known as the standard score) is a mathematical measurement that describes a value's relationship to the mean of a group of values, measured in terms of standard deviations from the mean. When applied to container resource tracking, the Z-score measures how far an instantaneous CPU percentage or memory usage value deviates from the container's recent historical baseline. A Z-score of zero indicates that the current resource consumption exactly matches the historical average, while a positive Z-score indicates how many standard deviations the current measurement lies above the mean.

```
       Normal Baseline Distribution                    Anomalous Resource Spike
       
                  Mean (μ)                                     Current Value (X)
                     |                                                 |
             .-------+-------.                                         |
           /                 \                                         |
          /                   \                                        v
  -------+---------------------+---------------------------------------+------> Z-Score
        -2σ          0        +2σ                                     +3.8σ
                                                                   (TRIGGER)
```

In mathematical terms, the Z-score \( Z \) for a current observation \( X \) is calculated using the sample mean \( \mu \) and standard deviation \( \sigma \):

\[
Z = \frac{X - \mu}{\sigma}
\]

To compute this in real time without storing massive arrays of historical metrics in memory, Dockture maintains a rolling window of recent resource statistics for each active container. As new resource usage samples are streamed from the Docker stats API, the anomaly detection engine continuously updates the rolling mean and variance. To prevent mathematical instability when resource metrics remain completely static (which would result in a standard deviation of zero and lead to division-by-zero errors), Dockture applies an adaptive variance floor parameter known as `anomaly_sensitivity`. This parameter clamps the minimum standard deviation denominator to a safe value, ensuring that subtle resource fluctuations during quiet operational periods do not trigger spurious anomaly alerts.

When the computed Z-score for a container's CPU or memory usage exceeds the configured `anomaly_threshold` (which defaults to a Z-score of 3.0, representing values beyond the 99.7th percentile of normal distribution), Dockture flags the event as a statistical anomaly. The daemon immediately constructs an anomaly warning payload containing the container's baseline average, standard deviation, peak resource value, and calculated Z-score. This warning is dispatched across configured notification channels, alerting system operators to runaway processes or memory leaks in real time before the host system runs out of memory or experiences unresponsive service hangs.

Users can fine-tune the behavior of the anomaly engine through the CLI configuration commands. Setting `anomaly_threshold` to higher values (such as 4.0 or 5.0) reduces sensitivity, ensuring that alerts are fired only during extreme resource spikes. Conversely, adjusting `anomaly_sensitivity` allows operators to customize how aggressively the engine reacts to low-variance services. If desired, anomaly detection can be toggled on or off globally per deployment environment, providing complete operational flexibility across development, staging, and production clusters.
