use tungstenite::connect;
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;

use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixStream,UnixListener};
use url::Url;
type TarotSocket = WebSocket<Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>;

pub fn start(str_socket: &str , str_websocket: &str) {
    let socket_file = std::path::Path::new(str_socket);
    if socket_file.exists() {
        println!("Removing dangling socket file {:?} ", socket_file);
        std::fs::remove_file(&socket_file).unwrap();
    }
    if let Ok(listener) = UnixListener::bind(socket_file) {
        println!("Bots listening to socket {}", str_socket);
        let mut code_count = 0;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let stream = BufReader::new(stream);
                    let url_websocket = format!("{}/new_new", str_websocket);
                    for line in stream.lines() {
                        let code = line.unwrap();
                        if code == "SHUTDOWN" {
                            break
                        } else {
                            let my_code_count = code_count.clone();
                            let my_url_websocket = url_websocket.clone();
                            std::thread::spawn(move || {
                                let nickname = format!("bot-{}", my_code_count);
                                let tarot_socket: TarotSocket = connect(Url::parse(&my_url_websocket)
                                    .unwrap())
                                    .expect("Can't connect")
                                    .0;
                                let mut bot = crate::player::SocketPlayer::new(tarot_socket, code.to_string(), nickname);
                                bot.play();
                                println!("a bot finished");
                            });
                        }
                        code_count = code_count + 1;
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
