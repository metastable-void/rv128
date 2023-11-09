# router-hello

```sh
cargo build --release
sudp cp target/release/rv128 /srv/router-hello
```

/srv/router-hello.env:

```sh
LISTEN_ADDR=[::]:80
ROUTER_ID=rt131
ADDRESS_V4=43.228.174.131
ADDRESS_V6=2001:df3:14c0:1131::1
```

Edit the env file as needed.

/etc/systemd/system/router-hello.service:

```systemd
[Unit]
Description=Router Hello Web Server
After=network.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=/srv
ExecStart=/srv/router-hello
Restart=on-failure
EnvironmentFile=/srv/router-hello.env

[Install]
WantedBy=multi-user.target
```

```sh
systemctl daemon-reload
systemctl restart router-hello
```
