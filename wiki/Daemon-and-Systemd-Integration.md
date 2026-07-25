# Systemd & Daemon Integration

Dockture runs as an unprivileged systemd user service on Linux host machines, or as a container via Docker Compose.

## Systemd User Service

To install and start the background service:

```bash
dockture service install
dockture service start
dockture service status
```

To keep the user service running after SSH logout, enable user linger:

```bash
loginctl enable-linger $USER
```

## Docker Compose

```yaml
version: "3.8"

services:
  dockture:
    image: engnhn/dockture:latest
    container_name: dockture
    restart: always
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ~/.config/dockture:/root/.config/dockture:ro
    environment:
      - DOCKTURE_CONFIG=/root/.config/dockture/config.toml
```

```bash
docker-compose up -d
```
