use std::sync::Arc;

use tokio::{io::WriteHalf, net::TcpStream, sync::RwLock};

use super::{TcpContextWriter, sessions_list::SessionList};

pub struct TcpServerSession {
    id: u128,
    data_writer: RwLock<TcpContextWriter>,
    session_list: Arc<SessionList>
}

impl TcpServerSession {
    pub fn new(id: u128, write_socket: WriteHalf<TcpStream>, list: Arc<SessionList>) -> TcpServerSession {
        TcpServerSession {
            id: id,
            data_writer: RwLock::new(TcpContextWriter::new(write_socket, id, list.clone())),
            session_list: list
        }
    }

    pub fn get_id(&self) -> u128{
        return self.id;
    }

    async fn send_and_hadle_error(&self, buff: &[u8]) -> bool {
        let mut write_access = self.data_writer.write().await;
        let result = write_access.send(buff).await;

        if let Err(err) = result {
            println!("error to send");
            return false;
        }

        true
    }

    pub async fn send(&self, buf: Vec<u8>) {        
        if !self
            .send_and_hadle_error(buf.as_slice())
            .await
        {
            self.disconnect().await;
        }
    }

    pub async fn disconnect(&self){
        let mut write_access = self.data_writer.write().await;
        write_access.disconnect().await;
    }
}