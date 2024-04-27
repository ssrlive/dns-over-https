# DNS-over-HTTPS

A lightweight DNS-over-HTTPS ("DOH") proxy written in Rust.

DNS-over-HTTPS is a lightweight proxy that will securely forward any requests to a DNS-over-HTTPS resolver such as [Cloudflare](https://developers.cloudflare.com/1.1.1.1/dns-over-https/).

**Current version:** 0.2.0  
**Supported Rust version:** 1.31

## Install

Download the latest binary for your architecture from the [releases page](https://github.com/ssrlive/dns-over-https/releases).

## Usage

```
Usage: dns-over-https [OPTIONS]

Options:
  -b, --bind <IP:port>       Listen for DNS requests on the addresses and ports [default: 127.0.0.1:53]
  -u, --upstream-urls <URL>  URL(s) of upstream DNS-over-HTTPS service [default: https://1.1.1.1/dns-query]
  -v, --verbosity <level>    Verbosity level [default: info] [possible values: off, error, warn, info, debug, trace]
      --service              Windows only: Run as a service
  -h, --help                 Print help
  -V, --version              Print version
```

### Running on Windows

To use DNS-over-HTTPS to encrypt your DNS requests on Windows, download and install the latest [release](https://github.com/ssrlive/dns-over-https/releases).

Or you can install it using `cargo`, assuming you have `Rust` installed:

```bash
cargo install dns-over-https
```
The binary will be installed in `C:\Users\<username>\.cargo\bin\dns-over-https.exe`.

Open a `powershell` terminal with administrative privileges and input command `New-Service` to create a new service,
- Name: `dns-over-https`
- BinaryPathName: `C:\Users\<username>\.cargo\bin\dns-over-https.exe -b 127.0.0.1:53 -b [::1]:53 --service`

![image](https://github.com/ssrlive/dns-over-https/assets/30760636/284e1063-179b-4ee9-8f85-1124769eb318)

Start the service: Input command `services` from start menu, open `Services` window, start the service `dns-over-https`.

![image](https://github.com/ssrlive/dns-over-https/assets/30760636/0c578370-e74e-43e5-9bdd-41701bd12d44)

Modify the DNS server address in the network adapter settings to `127.0.0.1`(IPv4) and `::1`(IPv6).

![image](https://github.com/ssrlive/dns-over-https/assets/30760636/25505389-ff61-44e7-88b1-6117eb36c66c)

Done.

### Running on a Pi-Hole

To use DNS-over-HTTPS to encrypt your DNS requests on a Pi-Hole, download and install the latest [release](https://github.com/ssrlive/dns-over-https/releases):

```console
pi@raspberrypi:~ $ wget https://github.com/ssrlive/dns-over-https/releases/download/v0.2.0/dns-over-https-v0.2.0-arm-unknown-linux-gnueabihf.tar.gz
pi@raspberrypi:~ $ tar xzf dns-over-https-v0.2.0-arm-unknown-linux-gnueabihf.tar.gz
pi@raspberrypi:~ $ sudo mv dns-over-https /usr/local/bin/
```

You can confirm dns-over-https is working properly by asking for the current version:

```console
pi@raspberrypi:~ $ dns-over-https --version
dns-over-https 0.2.0
```

You can then configure dns-over-https to run as a Systemd service that listens on port 5053 and forwards requests to [Cloudflare's DNS-over-HTTPS resolvers](https://developers.cloudflare.com/1.1.1.1/dns-over-https/).

First, create a system user for dns-over-https:

```console
pi@raspberrypi:~ $ sudo adduser --system --no-create-home dns-over-https
```

Then write out a Systemd unit file:

```console
pi@raspberrypi:~ $ sudo tee /lib/systemd/system/dns-over-https.service <<EOF
[Unit]
Description=dns-over-https
After=syslog.target network-online.target

[Service]
Type=simple
User=dns-over-https
ExecStart=/usr/local/bin/dns-over-https -b 127.0.0.1:5053 -u https://1.1.1.1/dns-query -u https://1.0.0.1/dns-query
Restart=on-failure
RestartSec=10
KillMode=process

[Install]
WantedBy=multi-user.target
EOF
```

You can now start up dns-over-https and check it is running:

```console
pi@raspberrypi:~ $ sudo systemctl enable dns-over-https
pi@raspberrypi:~ $ sudo systemctl start dns-over-https
pi@raspberrypi:~ $ sudo systemctl status dns-over-https
```

Finally, you can change your Pi-Hole configuration to use `127.0.0.1#5053` as its sole upstream DNS server and confirm your requests are now secure by using [Cloudflare's connection information page](https://1.1.1.1/help).

## References

* https://developers.cloudflare.com/1.1.1.1/dns-over-https/

## License

Copyright © 2024-2024 @ssrlive
Copyright © 2018-2019 Paul Mucur

Distributed under the MIT License.
