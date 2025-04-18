use tokio_tungstenite::tungstenite::{self, protocol::CloseFrame, Message};
use wscat_rs::slash::parse_slash_command;

#[test]
fn test_ping() {
    let msg = parse_slash_command("/ping hello").unwrap();
    assert_eq!(msg, Message::Ping(b"hello".to_vec().into()));
}

#[test]
fn test_unknown_command_falls_back() {
    assert!(parse_slash_command("/unknown blah").is_none());
}

#[test]
fn test_close_with_code_and_reason() {
    let msg = parse_slash_command("/close 1001 going away").unwrap();
    if let Message::Close(Some(CloseFrame { code, reason })) = msg {
        if !(code == tungstenite::protocol::frame::coding::CloseCode::Away) {
            panic!("expected CloseCode::Away(), got {:?}", code);
        };

        assert_eq!(reason, "going away");
    } else {
        panic!("expected Close message with frame");
    }
}
