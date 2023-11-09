# router-hello

```sh
cargo install router-hello
sudo router-hello install
sudo systemctl status router-hello
```

## Configuration

Configuration file is at `/etc/default/router-hello` by default.

By default the http server listens on all interfaces on port 80 (`[::]:80`).

Example:

```sh
LISTEN_ADDR=[::]:80
AS_NAME=MENHERA
ASN=AS63806
ROUTER_DOMAIN=nc.menhera.org
ROUTER_ID=rt131
ADDRESS_V4=43.228.174.131
ADDRESS_V6=2001:df3:14c0:1131::1
```

Edit this env file as needed.
