use anyhow::Result;
use atty::Stream;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use http::header::{HeaderName, HeaderValue};
use native_tls::TlsConnector;
use slash::parse_slash_command;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async_tls_with_config, Connector};

pub mod slash;

/// wscat‑style client with `wss://` support, `--insecure`,
/// optional slash-commands, and TTY‑aware prefixes.
#[derive(Parser)]
#[allow(clippy::struct_excessive_bools)]
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

    /// Custom headers to send with the WebSocket handshake
    #[clap(long = "header", value_parser = parse_key_val::<String, String>, number_of_values = 1)]
    headers: Vec<(String, String)>,

    /// Print request info before connecting
    #[clap(long)]
    verbose: bool,

    /// Print request and exit without connecting
    #[clap(long)]
    dry_run: bool,
}

fn parse_key_val<K, V>(s: &str) -> Result<(K, V), String>
where
    K: std::str::FromStr,
    V: std::str::FromStr,
    K::Err: ToString,
    V::Err: ToString,
{
    let pos = s
        .find(':')
        .ok_or_else(|| "invalid KEY:VALUE format".to_string())?;
    let key = s[..pos].trim().parse().map_err(|e: K::Err| e.to_string())?;
    let value = s[pos + 1..]
        .trim()
        .parse()
        .map_err(|e: V::Err| e.to_string())?;

    Ok((key, value))
}

#[tokio::main]
async fn main() -> Result<()> {
    let Opt {
        connect,
        insecure,
        slash,
        headers,
        verbose,
        dry_run,
    } = Opt::parse();

    // Convert URL string into a WebSocket request
    let mut request = connect.into_client_request()?;

    for (k, v) in &headers {
        let name = HeaderName::from_bytes(k.as_bytes())?;
        let value = HeaderValue::from_str(v)?;
        request.headers_mut().append(name, value);
    }

    if verbose || dry_run {
        println!("Connecting to: {}", request.uri());
        println!("Request Headers:");
        for (name, value) in request.headers() {
            println!("  {}: {}", name, value.to_str().unwrap_or("<invalid utf8>"));
        }
    }

    if dry_run {
        println!("Dry run enabled — exiting before connection.");
        return Ok(());
    }
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

    let stdin_task = tokio::spawn(async move {
        let mut lines = io::BufReader::new(io::stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let msg = if slash && line.starts_with('/') {
                parse_slash_command(&line).unwrap_or_else(|| Message::Text(line.clone().into()))
            } else {
                Message::Text(line.into())
            };
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Task: WebSocket → stdout, TTY‑aware prefixing
    //
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
