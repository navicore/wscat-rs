[![Dependabot Updates](https://github.com/navicore/wscat-rs/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/navicore/wscat-rs/actions/workflows/dependabot/dependabot-updates) [![Rust](https://github.com/navicore/wscat-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/navicore/wscat-rs/actions/workflows/rust.yml) [![rust-clippy analyze](https://github.com/navicore/wscat-rs/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/navicore/wscat-rs/actions/workflows/rust-clippy.yml)

wscat-rs
===========

A re-implementation of https://github.com/websockets/wscat for my own amazement and convenience.

```bash
wscat‑style client with `wss://` support, `--insecure`, optional slash-commands, and TTY‑aware prefixes

Usage: wscat-rs [OPTIONS] --connect <CONNECT>

Options:
  -c, --connect <CONNECT>  WebSocket URL to connect (ws:// or wss://)
      --insecure           Skip TLS certificate validation
      --slash              Enable slash commands (/ping, /pong, /close)
  -h, --help               Print help
```
