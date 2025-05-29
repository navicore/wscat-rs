//! Slash command parsing for interactive WebSocket control.
//! 
//! This module provides functionality to parse slash commands that allow
//! users to send WebSocket control frames interactively.

use tokio_tungstenite::tungstenite::protocol::{frame::coding::CloseCode, CloseFrame};
use tokio_tungstenite::tungstenite::Message;

/// Parses a slash command string into a WebSocket message.
/// 
/// Supported commands:
/// - `/ping [data]` - Sends a WebSocket ping frame with optional data
/// - `/pong [data]` - Sends a WebSocket pong frame with optional data
/// - `/close [code] [reason]` - Sends a close frame with optional code and reason
/// 
/// # Arguments
/// 
/// * `line` - The input line to parse, should start with a slash
/// 
/// # Returns
/// 
/// * `Some(Message)` if a valid slash command was parsed
/// * `None` if the line doesn't match any known slash command
/// 
/// # Examples
/// 
/// ```rust
/// use wscat_rs::parse_slash_command;
/// use tokio_tungstenite::tungstenite::Message;
/// 
/// // Send a ping
/// if let Some(Message::Ping(data)) = parse_slash_command("/ping hello") {
///     assert_eq!(data.as_ref(), b"hello");
/// }
/// 
/// // Send a close with code and reason
/// if let Some(Message::Close(Some(frame))) = parse_slash_command("/close 1001 going away") {
///     assert_eq!(frame.code, tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Away);
///     assert_eq!(frame.reason, "going away");
/// }
/// 
/// // Unknown commands return None
/// assert!(parse_slash_command("/unknown").is_none());
/// ```
#[must_use]
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
