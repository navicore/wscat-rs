use clap::Parser;
use futures::{SinkExt, StreamExt};
use tokio::io::{self, AsyncBufReadExt};
use tokio_tungstenite::connect_async;
use tungstenite::Message;
/// a rust re-imlementation of a simple WebSocket client https://github.com/websockets/wscat
#[derive(Parser)]
struct Opt {
    /// Connect to this WebSocket URL
    #[clap(short, long)]
    connect: String,
    /// Don’t print colors
    #[clap(long)]
    no_color: bool,
    // … add more flags as needed …
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Opt {
        connect,
        no_color: _,
    } = Opt::parse();

    // Establish WebSocket connection
    let (ws_stream, _) = connect_async(&connect).await?;
    let (mut write, mut read) = ws_stream.split();

    // Task: read from stdin and send to socket
    let stdin_task = tokio::spawn(async move {
        let mut lines = io::BufReader::new(io::stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            write.send(Message::Text(line.into())).await.ok();
        }
    });

    // Task: read from socket and print to stdout
    let socket_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(txt) = msg {
                println!("< {}", txt);
            }
        }
    });

    // Await both (will exit when either ends)
    let _ = tokio::try_join!(stdin_task, socket_task)?;
    Ok(())
}
