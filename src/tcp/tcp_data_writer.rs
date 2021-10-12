use std::sync::Arc;

use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::{TcpStream}
};

use super::sessions_list::{SessionList};


pub struct TcpContextWriter {
    tcp_stream: Option<WriteHalf<TcpStream>>,
    id: u128,
    sessions_list: Arc<SessionList>
}

impl TcpContextWriter {
    pub fn new(tcp_stream: WriteHalf<TcpStream>, id: u128, list: Arc<SessionList>) -> TcpContextWriter {
        TcpContextWriter {
            tcp_stream: Some(tcp_stream),
            id: id,
            sessions_list: list
        }
    }

    pub async fn send(&mut self, buf: &[u8]) -> Result<(), String> {
        match self.tcp_stream.as_mut() {
            Some(tcp_stream) => {
                let result = tcp_stream.write_all(buf).await;

                if let Err(err) = result {
                    return Err(format!(
                        "Can not send to the socket {}. Err:{}",
                        self.id, err
                    ));
                } else {
                    return Ok(());
                }
            }
            None => {
                return Err(format!("Socket {:?} is disconnected", self.id));
            }
        }
    }

    pub async fn disconnect(&mut self) {
        if self.tcp_stream.is_none() {
            return;
        }

        let mut tcp_stream = None;

        std::mem::swap(&mut tcp_stream, &mut self.tcp_stream);

        let result = tcp_stream.unwrap().shutdown().await;

        if let Err(err) = result {
            return println!("Can nod disconnect tcp socket{:?}. Err: {:?}", self.id, err);
        }
    }
}