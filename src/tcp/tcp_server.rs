use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf}, net::{TcpListener, TcpStream}, sync::RwLock};


use super::{TcpServerSession, buff_reader::BufferReader, connection::handle_incoming_payload, sessions_list::SessionList};

pub type ConnectionId = u128;

const PING: &str = "PING";
const PONG: &str = "PONG";
const MESSAGE_SPLITTER: [u8; 2] = [13, 10];


pub async fn start(addr: SocketAddr, list: Arc<SessionList>) {
    let listener = TcpListener::bind(addr).await.unwrap();

    let mut socket_id: ConnectionId = 0;

    loop {
        let accep_socket = listener.accept().await;

        if accep_socket.is_err() {
            println!("Failed to load connection");
        }

        let (socket, _) = accep_socket.unwrap();
        let (reed_socket, write_socket) = io::split(socket);
        socket_id += 1;

        let tcp_session = Arc::new(TcpServerSession::new(socket_id, write_socket, list.clone()));
        list.add_session(tcp_session.clone()).await;
        

        tokio::task::spawn(process_socket(reed_socket, tcp_session, list.clone()));

    }
}


async fn process_socket(
    read_socket: ReadHalf<TcpStream>,
    tcp_session: Arc<TcpServerSession>,
    list: Arc<SessionList>
) {
    let socket_loop_result =
        tokio::task::spawn(socket_loop(read_socket, tcp_session.clone())).await;


    if let Err(err) = socket_loop_result {
       println!("Disconect");
    } 

    list.remove_session(tcp_session.get_id()).await
}


async fn socket_loop(
    mut read_socket: ReadHalf<TcpStream>,
    session: Arc<TcpServerSession>,
) -> Result<(), ()> {

    let mut bf_reader = BufferReader::new();

    loop {
        let mut reed_buffer = [0; 250];
        read_socket.read(&mut reed_buffer).await.unwrap();
        bf_reader.write_to_buff(reed_buffer.to_vec());

        let reed_result = bf_reader.get_next_package_if_exists(&MESSAGE_SPLITTER.to_vec());

        if reed_result.is_none() {
            continue;
        }

        handle_incoming_payload(reed_result.unwrap(), &session).await;



    }
}
