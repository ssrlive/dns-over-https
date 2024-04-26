# DNS-over-TLS

A lightweight DNS-over-HTTPS ("DOH") proxy written in Rust.

DNS-over-TLS is a lightweight proxy that will securely forward any requests to a DNS-over-HTTPS resolver such as [Cloudflare](https://developers.cloudflare.com/1.1.1.1/dns-over-https/).

**Current version:** 0.2.0  
**Supported Rust version:** 1.31

## Install

Download the latest binary for your architecture from the [releases page](https://github.com/ssrlive/dns-over-tls/releases).

## Usage

```
Usage: dns-over-tls.exe [OPTIONS]

Options:
  -b, --bind <IP:port>       Listen for DNS requests on the addresses and ports [default: 127.0.0.1:53]
  -u, --upstream-urls <URL>  URL(s) of upstream DNS-over-HTTPS service [default: https://1.1.1.1/dns-query]
  -v, --verbosity <level>    Verbosity level [default: info] [possible values: off, error, warn, info, debug, trace]
  -h, --help                 Print help
  -V, --version              Print version
```

### Running on a Pi-Hole

To use DNS-over-TLS to encrypt your DNS requests on a Pi-Hole, download and install the latest [release](https://github.com/ssrlive/dns-over-tls/releases):

```console
pi@raspberrypi:~ $ wget https://github.com/ssrlive/dns-over-tls/releases/download/v0.2.0/dns-over-tls-v0.2.0-arm-unknown-linux-gnueabihf.tar.gz
pi@raspberrypi:~ $ tar xzf dns-over-tls-v0.2.0-arm-unknown-linux-gnueabihf.tar.gz
pi@raspberrypi:~ $ sudo mv dns-over-tls /usr/local/bin/
```

You can confirm dns-over-tls is working properly by asking for the current version:

```console
pi@raspberrypi:~ $ dns-over-tls --version
dns-over-tls 0.2.0
```

You can then configure dns-over-tls to run as a Systemd service that listens on port 5053 and forwards requests to [Cloudflare's DNS-over-HTTPS resolvers](https://developers.cloudflare.com/1.1.1.1/dns-over-https/).

First, create a system user for dns-over-tls:

```console
pi@raspberrypi:~ $ sudo adduser --system --no-create-home dns-over-tls
```

Then write out a Systemd unit file:

```console
pi@raspberrypi:~ $ sudo tee /lib/systemd/system/dns-over-tls.service <<EOF
[Unit]
Description=dns-over-tls
After=syslog.target network-online.target

[Service]
Type=simple
User=dns-over-tls
ExecStart=/usr/local/bin/dns-over-tls -b 127.0.0.1:5053 -u https://1.1.1.1/dns-query -u https://1.0.0.1/dns-query
Restart=on-failure
RestartSec=10
KillMode=process

[Install]
WantedBy=multi-user.target
EOF
```

You can now start up dns-over-tls and check it is running:

```console
pi@raspberrypi:~ $ sudo systemctl enable dns-over-tls
pi@raspberrypi:~ $ sudo systemctl start dns-over-tls
pi@raspberrypi:~ $ sudo systemctl status dns-over-tls
```

Finally, you can change your Pi-Hole configuration to use `127.0.0.1#5053` as its sole upstream DNS server and confirm your requests are now secure by using [Cloudflare's connection information page](https://1.1.1.1/help).

## References

* https://developers.cloudflare.com/1.1.1.1/dns-over-https/

## License

Copyright © 2024-2024 @ssrlive
Copyright © 2018-2019 Paul Mucur

Distributed under the MIT License.
