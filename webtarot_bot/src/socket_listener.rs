use tungstenite::connect;
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixStream,UnixListener};
use url::Url;
type TarotSocket = WebSocket<Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>;

struct NickNamer {
    parties: HashMap<String, u8>,
}

impl NickNamer {
    pub fn new() -> Self {
        NickNamer { parties: HashMap::new() }
    }

    pub fn get_nickname(&mut self, code: &str) -> String {
        *self.parties.entry(code.to_string()).or_insert(0) += 1;
        format!("bot{}", self.parties[code])
    }

    pub fn delete_party(&mut self, code: &str) {
        self.parties.remove(code);
    }
}

pub fn start(str_socket: &str , str_websocket: &str) {
    let socket_file = std::path::Path::new(str_socket);
    if socket_file.exists() {
        // remove dangling socket file
        std::fs::remove_file(&socket_file).unwrap();
    }
    if let Ok(listener) = UnixListener::bind(socket_file) {
        println!("Bots listening to socket {}", str_socket);
        let nicknamer = Mutex::new(NickNamer::new()); 
        let nicknamer = Arc::new(Mutex::new(NickNamer::new())); 
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let stream = BufReader::new(stream);
                    let url_websocket = format!("{}/ws/new_new", str_websocket);
                    for line in stream.lines() {
                        let code = line.unwrap();
                        if code == "SHUTDOWN" {
                            break
                        } else {
                            let my_url_websocket = url_websocket.clone();
                            let my_nicknamer = nicknamer.clone();
                            std::thread::spawn(move || {
                                let nickname = my_nicknamer.lock().unwrap().get_nickname(&code);
                                let tarot_socket: TarotSocket = connect(Url::parse(&my_url_websocket)
                                    .unwrap())
                                    .expect("Can't connect")
                                    .0;
                                let mut bot = crate::player::SocketPlayer::new(tarot_socket, code.to_string(), nickname);
                                bot.play();
                                // we clean the nicknamer as soon as the first bot quits the party
                                my_nicknamer.lock().unwrap().delete_party(&code);
                                // println!("a bot finished");
                            });
                        }
                    }
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
        println!("Shuting down bot listener");
        drop(listener);
        if socket_file.exists() {
            std::fs::remove_file(&socket_file).unwrap();
        }
    } else {
        println!("couldn't connect to socket {}", str_socket);
    }
}
