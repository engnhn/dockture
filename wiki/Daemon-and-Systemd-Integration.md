# Daemon and Systemd Integration Guide

Running monitoring tools continuously in production environments requires reliable process supervision, automatic restart capabilities upon host reboots, and clean background daemon execution. On modern Linux operating systems, systemd serves as the standard service manager for orchestrating background processes. Dockture seamlessly integrates with systemd user services, allowing operators to deploy, manage, and inspect the monitoring daemon without needing root privileges or complex manual unit file creation.

Unlike system-level systemd services that operate globally under root privileges, systemd user services run within the unprivileged context of a specific user session. This model aligns perfectly with security best practices by preventing the monitoring process from acquiring elevated system privileges unnecessarily while still granting access to the local Docker socket. When Dockture is deployed as a systemd user service, systemd supervises the daemon process, automatically restarting it if it encounters unexpected failures and initializing the monitoring service automatically upon host startup when linger is enabled.

```
+-----------------------------------------------------------------------------------+
|                                 Host Operating System                             |
|                                                                                   |
|  +-----------------------------------------------------------------------------+  |
|  |                     Systemd User Service Manager                            |  |
|  |                   (systemctl --user status dockture)                        |  |
|  |                                                                             |  |
|  |   Unit File: ~/.config/systemd/user/dockture.service                       |  |
|  |   Restart Policy: on-failure                                                |  |
|  |   RestartSec: 5s                                                            |  |
|  |                                                                             |  |
|  |   +---------------------------------------------------------------------+   |  |
|  |   |                         Supervised Process                          |   |  |
|  |   |                     dockture run (Daemon Engine)                    |   |  |
|  |   +---------------------------------------------------------------------+   |  |
|  +-----------------------------------------------------------------------------+  |
|                                                                                   |
+-----------------------------------------------------------------------------------+
```

Dockture incorporates built-in service management capabilities directly into its CLI interface via the `dockture service` subcommand family. Operators do not need to write systemd unit definitions manually. Running `dockture service install` automatically generates a fully configured unit file placed at `~/.config/systemd/user/dockture.service`. The generated unit file explicitly defines the path to the compiled binary, sets execution flags for background daemon operation (`dockture run`), specifies automated restart policies (`Restart=on-failure`), and configures appropriate environment variables.

Managing the lifecycle of the background daemon is performed effortlessly using the CLI subcommands. After installation, running `dockture service start` instructs the systemd user manager to launch the monitoring daemon immediately. Subsequent lifecycle operations—such as stopping the service (`dockture service stop`), restarting after configuration changes (`dockture service restart`), or checking real-time execution status (`dockture service status`)—can be executed directly through Dockture without needing to remember detailed `systemctl --user` flags.

```bash
# Install and register the systemd user unit file
dockture service install

# Start the background daemon service
dockture service start

# Query the operational status of the service
dockture service status

# Stop and uninstall the service when needed
dockture service stop
dockture service uninstall
```

For host environments where user sessions terminate upon logout, systemd can be configured to keep user services running persistently across user disconnections by enabling user lingering. Executing `loginctl enable-linger $USER` ensures that systemd user manager instances start during host boot up and remain active continuously, enabling Dockture to provide uninterrupted 24/7 container monitoring regardless of interactive user login sessions.

---

## Alternative Production Deployment: Docker & Docker Compose

For containerized infrastructure where running native binaries outside of container runtimes is restricted, Dockture can be deployed as a minimal Docker container.

### `docker-compose.yml` Specification

```yaml
version: "3.8"

services:
  dockture:
    image: sampletheory/dockture:latest
    container_name: dockture-daemon
    restart: always
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ~/.config/dockture:/root/.config/dockture:ro
    environment:
      - DOCKTURE_CONFIG=/root/.config/dockture/config.toml
```

To launch via Docker Compose:
```bash
docker-compose up -d
```

---

## Production Deployment Comparison & Memory Footprint

| Deployment Mode | RAM Footprint (RSS) | Installation Method | Best For |
|---|---|---|---|
| **Native Systemd Service (Recommended)** | **~1.0 MB RAM** | `curl -fsSL .../install.sh \| sh` + `dockture service install` | Bare-metal servers, VMs, edge nodes requiring minimal overhead. |
| **Docker Container** | **~2.0 MB RAM** | `docker-compose up -d` | Fully containerized environments, Kubernetes, or immutable OS hosts. |

