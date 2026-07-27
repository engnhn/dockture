# daemon and systemd integration

dockture can run as an unprivileged systemd user service on linux hosts or as a container managed by docker compose. when installed via CLI (`dockture service install`), dockture writes a systemd user unit file to `~/.config/systemd/user/dockture.service`, reloads the user daemon, and enables background execution:

```ini
[Unit]
Description=dockture container monitoring service
After=network.target docker.service

[Service]
Type=simple
ExecStart=/usr/local/bin/dockture run
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
```

the user service is controlled using `dockture service <action>` subcommands (`install`, `start`, `stop`, `restart`, `status`, `uninstall`). by default, systemd user services terminate when a user closes their SSH session; to keep dockture monitoring continuously after logout, enable linger mode with `loginctl enable-linger $USER`.

for containerized deployments, dockture can be run with docker compose by mounting the host docker socket and configuration directory:

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

start the stack with `docker-compose up -d` and inspect daemon logs using `docker logs -f dockture`.

---

previous: [notification channels and alerting](./Notification-Channels-and-Alerting.md) | home: [home](./Home.md) | next: [cli reference and operations](./CLI-Reference-and-Operations.md)
