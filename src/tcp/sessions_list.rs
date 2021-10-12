use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use super::TcpServerSession;

pub struct SessionList{
    data: RwLock<HashMap<u128, Arc<TcpServerSession>>>
}

impl SessionList {

    pub fn new() -> SessionList{
        SessionList{
            data: RwLock::new(HashMap::new())
        }
    }

    pub async fn send_to_all(&self, data_to_send: Vec<u8>){
        let read_data = self.data.read().await;
        for (_, read) in read_data.iter() {
            read.send(data_to_send.clone()).await;
        }
    }

    pub async fn remove_session(&self, id_to_delete: u128){
        let mut write_access = self.data.write().await;
        write_access.remove(&id_to_delete);
    }

    pub async fn add_session(&self, session: Arc<TcpServerSession>){
        let mut write_access = self.data.write().await;
        write_access.insert(session.get_id(), session);
    }
}