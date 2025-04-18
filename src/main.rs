use anyhow::Result;
use atty::Stream;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use native_tls::TlsConnector;
use tokio::io::AsyncBufReadExt;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async_tls_with_config, Connector};

/// wscat‑style client with `wss://` support and `--insecure`.
#[derive(Parser)]
struct Opt {
    /// WebSocket URL to connect (ws:// or wss://)
    #[clap(short, long)]
    connect: String,

    /// Skip TLS certificate validation
    #[clap(long)]
    insecure: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Opt { connect, insecure } = Opt::parse();

    // Turn the URL string into a Request
    let request = connect.into_client_request()?;

    // Build the native_tls connector
    let mut builder = TlsConnector::builder();
    if insecure {
        builder.danger_accept_invalid_certs(true);
    }
    let native_conn = builder.build()?;

    // Wrap it in tokio-tungstenite's Connector enum
    let tls_connector = Some(Connector::NativeTls(native_conn));

    // Detect if stdout is a TTY for prefixing
    let prefix_enabled = atty::is(Stream::Stdout);

    // Dial out, using TLS if scheme == wss://
    let (ws_stream, _) = connect_async_tls_with_config(request, None, tls_connector).await?;
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

    let (mut sink, mut stream) = ws_stream.split();

    // Task: stdin → WebSocket
    let stdin_task = tokio::spawn(async move {
        let mut lines = tokio::io::BufReader::new(tokio::io::stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if sink.send(Message::Text(line)).await.is_err() {
                break;
            }
        }
    });

    // Task: WebSocket → stdout
    let socket_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(t) => {
                    if prefix_enabled {
                        println!("< {}", t);
                    } else {
                        println!("{}", t);
                    }
                }
                Message::Binary(b) => {
                    if prefix_enabled {
                        println!("< [binary: {} bytes]", b.len());
                    } else {
                        // print raw bytes as string fallback
                        println!("{}", String::from_utf8_lossy(&b));
                    }
                }
                Message::Close(c) => {
                    if prefix_enabled {
                        println!("< closed: {:?}", c);
                    } else {
                        eprintln!("closed: {:?}", c);
                    }
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either side to finish
    let _ = tokio::try_join!(stdin_task, socket_task)?;
    Ok(())
}
