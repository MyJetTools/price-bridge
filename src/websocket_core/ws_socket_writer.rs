use futures::SinkExt;
use tokio::sync::RwLock;
use tokio_tungstenite::{tungstenite::Message};
use futures::stream::SplitSink;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio::net::TcpStream;

pub struct WsMessageWriter{
    pub send_sing: RwLock<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>
}

impl WsMessageWriter {
    pub fn new(send: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) -> WsMessageWriter{
        WsMessageWriter{
            send_sing: RwLock::new(send)
        }
    }

    pub async fn send_data(&self, mess: Message){
        let mut read_access = self.send_sing.write().await;
        read_access.send(mess).await.expect("Cant send message to ws.");
    }
}