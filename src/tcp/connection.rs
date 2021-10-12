use super::TcpServerSession;

const PING: &str = "PING";
const PONG: &str = "PONG";

pub async fn handle_incoming_payload(
    tcp_data: Vec<u8>,
    session: &TcpServerSession
) -> Result<(), ()> {
    let message = std::str::from_utf8(&tcp_data).unwrap();
    
    if message == PING {
        session.send(PONG.as_bytes().to_vec()).await
    }
    return Ok(());
}
