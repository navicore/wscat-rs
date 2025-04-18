use core::panic;

use anyhow::Result;
use atty::Stream;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use native_tls::TlsConnector;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async_tls_with_config, Connector};

/// wscat‑style client with `wss://` support, `--insecure`,
/// optional slash-commands, and TTY‑aware prefixes.
#[derive(Parser)]
struct Opt {
    /// WebSocket URL to connect (ws:// or wss://)
    #[clap(short, long)]
    connect: String,

    /// Skip TLS certificate validation
    #[clap(long)]
    insecure: bool,

    /// Enable slash commands (/ping, /pong, /close)
    #[clap(long)]
    slash: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Opt {
        connect,
        insecure,
        slash,
    } = Opt::parse();

    // Convert URL string into a WebSocket request
    let request = connect.into_client_request()?;

    // Build the native-tls connector
    let mut builder = TlsConnector::builder();
    if insecure {
        builder.danger_accept_invalid_certs(true);
    }
    let native_conn = builder.build()?;

    // Wrap in tokio-tungstenite Connector
    let tls_connector = Some(Connector::NativeTls(native_conn));

    // Will prefix messages only if stdout is a TTY
    let prefix_enabled = atty::is(Stream::Stdout);

    // Dial the WebSocket (uses TLS for wss://)
    let (ws_stream, _) = connect_async_tls_with_config(request, None, false, tls_connector).await?;
    let (mut sink, mut stream) = ws_stream.split();

    // Task: stdin → WebSocket, with optional slash-commands
    let stdin_task = tokio::spawn(async move {
        let mut lines = io::BufReader::new(io::stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if slash && line.starts_with('/') {
                // handle slash commands
                let mut parts = line.splitn(2, ' ');
                // panic if no command is provided
                let cmd = parts
                    .next()
                    .unwrap_or_else(|| panic!("Slash command expected"));
                let rest = parts.next().unwrap_or("");
                match cmd {
                    "/ping" => {
                        let _ = sink
                            .send(Message::Ping(rest.as_bytes().to_vec().into()))
                            .await;
                    }
                    "/pong" => {
                        let _ = sink
                            .send(Message::Pong(rest.as_bytes().to_vec().into()))
                            .await;
                    }
                    "/close" => {
                        let mut sub = rest.splitn(2, ' ');
                        let code = sub
                            .next()
                            .and_then(|c| c.parse::<u16>().ok())
                            .unwrap_or(1000);
                        let reason = sub.next().unwrap_or("").to_string();
                        let close_frame = CloseFrame {
                            code: CloseCode::from(code),
                            reason: reason.into(),
                        };
                        let _ = sink.send(Message::Close(Some(close_frame))).await;
                    }
                    _ => {
                        // unknown slash, send raw text
                        let _ = sink.send(Message::Text(line.clone().into())).await;
                    }
                }
            } else if sink.send(Message::Text(line.into())).await.is_err() {
                break;
            }
        }
    });

    // Task: WebSocket → stdout, TTY‑aware prefixing
    let socket_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(t) => {
                    if prefix_enabled {
                        println!("< {t}");
                    } else {
                        println!("{t}");
                    }
                }
                Message::Binary(b) => {
                    if prefix_enabled {
                        println!("< [binary: {} bytes]", b.len());
                    } else {
                        // raw binary fallback as UTF-8
                        println!("{}", String::from_utf8_lossy(&b));
                    }
                }
                Message::Close(cf) => {
                    if prefix_enabled {
                        println!("< closed: {cf:?}");
                    } else {
                        eprintln!("closed: {cf:?}");
                    }
                    break;
                }
                _ => {}
            }
        }
    });

    let _ = tokio::try_join!(stdin_task, socket_task)?;
    Ok(())
}
