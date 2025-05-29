//! # wscat-rs
//! 
//! A WebSocket client for the command line, similar to `netcat` but for WebSocket connections.
//! 
//! This crate provides a CLI tool that can connect to WebSocket servers (both `ws://` and `wss://`),
//! send and receive messages interactively, and optionally use slash commands for WebSocket-specific
//! operations like ping/pong and connection closing.
//! 
//! ## Features
//! 
//! - Connect to WebSocket servers using `ws://` or `wss://` protocols
//! - Skip TLS certificate validation with `--insecure` flag
//! - Send custom headers with the initial WebSocket handshake
//! - Specify WebSocket subprotocols
//! - Interactive slash commands (`/ping`, `/pong`, `/close`)
//! - TTY-aware output formatting (prefixes messages with `<` when stdout is a terminal)
//! - Verbose mode to see request details
//! - Dry-run mode to preview the connection without actually connecting
//! 
//! ## Usage
//! 
//! Basic connection:
//! ```bash
//! wscat-rs -c wss://echo.websocket.org
//! ```
//! 
//! With custom headers and subprotocol:
//! ```bash
//! wscat-rs -c wss://example.com --header "Authorization: Bearer token" --protocol graphql-ws
//! ```
//! 
//! With slash commands enabled:
//! ```bash
//! wscat-rs -c wss://example.com --slash
//! # Then in the session:
//! # /ping hello
//! # /close 1000 goodbye
//! ```
//! 
//! ## Library Usage
//! 
//! While primarily designed as a CLI tool, the slash command parsing functionality
//! is exposed for library use:
//! 
//! ```rust
//! use wscat_rs::parse_slash_command;
//! use tokio_tungstenite::tungstenite::Message;
//! 
//! if let Some(msg) = parse_slash_command("/ping hello") {
//!     match msg {
//!         Message::Ping(data) => println!("Ping with data: {:?}", data),
//!         _ => {}
//!     }
//! }
//! ```

pub mod slash;

pub use slash::parse_slash_command;
