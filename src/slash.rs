use tokio_tungstenite::tungstenite::protocol::{frame::coding::CloseCode, CloseFrame};
use tokio_tungstenite::tungstenite::Message;

pub fn parse_slash_command(line: &str) -> Option<Message> {
    let mut parts = line.splitn(2, ' ');
    let cmd = parts.next()?;
    let rest = parts.next().unwrap_or("");

    match cmd {
        "/ping" => Some(Message::Ping(rest.as_bytes().to_vec().into())),
        "/pong" => Some(Message::Pong(rest.as_bytes().to_vec().into())),
        "/close" => {
            let mut sub = rest.splitn(2, ' ');
            let code = sub
                .next()
                .and_then(|c| c.parse::<u16>().ok())
                .unwrap_or(1000);
            let reason = sub.next().unwrap_or("").to_string();
            let frame = CloseFrame {
                code: CloseCode::from(code),
                reason: reason.into(),
            };
            Some(Message::Close(Some(frame)))
        }
        _ => None,
    }
}
