// use std::{collections::HashMap, sync::atomic::AtomicBool, time::Duration};
// use std::{net::SocketAddr};
// use super::BufferReader;
// use chrono::Utc;
// use tokio::{
//     io::{self, AsyncReadExt, AsyncWriteExt, WriteHalf},
//     net::{TcpListener, TcpStream},
//     sync::{
//         mpsc::{UnboundedReceiver, UnboundedSender},
//         RwLock,
//     },
//     time::sleep,
// };

// const PING: &str = "PING";
// const PONG: &str = "PONG";
// const MESSAGE_SPLITTER: [u8; 2] = [13, 10];

// struct ClientSocket {
//     buffer: BufferReader,
//     stream: TcpStream,
//     is_handling_events: AtomicBool,
//     message_handle: UnboundedReceiver<Vec<u8>>,
//     message_sender: UnboundedSender<Vec<u8>>,
//     messages_to_send_handler: UnboundedReceiver<Vec<u8>>,
//     messages_to_send_sender: UnboundedSender<Vec<u8>>,
// }

// impl ClientSocket {
//     fn new(stream: TcpStream) -> ClientSocket {
//         let (input_tx, input_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
//         let (output_tx, output_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

//         ClientSocket {
//             buffer: BufferReader::new(),
//             stream: stream,
//             is_handling_events: AtomicBool::new(false),
//             messages_to_send_handler: output_rx,
//             messages_to_send_sender: output_tx,
//             message_handle: input_rx,
//             message_sender: input_tx,
//         }
//     }

//     async fn handle_message_to_send(&mut self, message_to_send: Vec<u8>) {
//         self.messages_to_send_sender.send(message_to_send).unwrap();
//     }

//     async fn send_messages(&mut self) {
//         loop {
//             let message = self.messages_to_send_handler.recv().await;
//             println!("Message send");

//             match message {
//                 Some(mut mess) => {
//                     mess.extend_from_slice(&MESSAGE_SPLITTER);
//                     self.stream.write_all(&mess).await.unwrap();
//                 }
//                 None => sleep(Duration::from_millis(10)).await,
//             }
//         }
//     }

//     async fn process_messages_from_client(&mut self) {
//         loop {
//             let message = self.message_handle.recv().await;

//             match message {
//                 Some(data) => {
//                     let string_mess = std::str::from_utf8(data.as_slice()).unwrap();

//                     if string_mess == PONG {
//                         self.is_handling_events
//                             .swap(true, std::sync::atomic::Ordering::Relaxed);
//                         println!(
//                             "Reserve message from client, it is PONG. Message: {}",
//                             string_mess
//                         );
//                         return;
//                     }

//                     println!(
//                         "Reserve message from client, but it is not a PONG. Message: {}",
//                         string_mess
//                     );
//                 }
//                 None => sleep(Duration::from_millis(100)).await,
//             }
//         }
//     }

//     async fn reed_next_message(&mut self) {
//         loop {
//             let message = self
//                 .buffer
//                 .get_next_package_if_exists(&MESSAGE_SPLITTER.to_vec());

//             if message.is_some() {
//                 self.message_sender.send(message.unwrap()).unwrap();
//             }

//             let mut reed_buffer = [0; 255];
//             self.stream.read(&mut reed_buffer).await.unwrap();
//             self.buffer.write_to_buff(reed_buffer.to_vec());

//             let message = self
//                 .buffer
//                 .get_next_package_if_exists(&MESSAGE_SPLITTER.to_vec());

//             if message.is_some() {
//                 self.message_sender.send(message.unwrap()).unwrap();
//             }
//         }
//     }
// }

// // let ping_client_socket = client_socket.clone();
// // let send_client_socket = client_socket.clone();
// // let handle_client_socket = client_socket.clone();

// pub struct EventTcpServer {
//     write: RwLock<HashMap<i64, RwLock<WriteHalf<TcpStream>>>>,
// }

// impl EventTcpServer {
//     pub fn new() -> EventTcpServer {
//         EventTcpServer {
//             write: RwLock::new(HashMap::new()),
//         }
//     }

//     pub async fn start(&self) {
//         let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], 8080))).await.unwrap();
//         loop {
//             let (socket, _) = listener.accept().await.unwrap();
//             let (socker_reed, socker_write) = io::split(socket);

//             let socket_id = Utc::now();

//             self.write
//                 .write()
//                 .await
//                 .insert(socket_id.timestamp_millis(), RwLock::new(socker_write));

//             println!("Added connection");
//         }
//     }

//     pub async fn send_event_to_all_sockets(&self, mut message: Vec<u8>) {
//         message.extend_from_slice(&MESSAGE_SPLITTER);

//         for (id, stream) in self.write.read().await.iter() {
//             let send_result = stream.write().await.write_all(message.as_slice()).await;

//             if !send_result.is_ok() {
//                 println!("cant send to socket. Remove: {}", id);
//             }
//         }
//     }
// }
