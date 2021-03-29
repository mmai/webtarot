use clap::{Arg, App};

mod explorer;
mod player;
mod player_factory;

use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixStream,UnixListener};

use tungstenite::connect;
use tungstenite::stream::Stream;
use tungstenite::protocol::WebSocket;
// use rayon::prelude::*;
use url::Url;
type TarotSocket = WebSocket<Stream<std::net::TcpStream, native_tls::TlsStream<std::net::TcpStream>>>;

pub fn main() {
    let version = format!("{}.{}.{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));
    // let author = format!("{}", env!("CARGO_PKG_AUTHORS"));
    let author = env!("CARGO_PKG_AUTHORS");
    // let name = format!("{}", env!("CARGO_PKG_NAME"));
    let name = env!("CARGO_PKG_NAME");

    let app = App::new("Webtarot Bot")
        .version(version.as_str())
        .author(author)
        .about(name)
        .arg(Arg::with_name("socket")
             .short("l")
             .long("socket")
             .help("socket file accepting requests")
             .takes_value(true))
        .arg(Arg::with_name("command")
             .short("c")
             .long("command")
             .value_name("COMMAND")
             .help("Command to execute")
             .takes_value(true))
        .arg(Arg::with_name("joincode")
             .short("j")
             .long("join_code")
             .value_name("JOINCODE")
             .help("Game join code")
             .takes_value(true))
        .arg(Arg::with_name("websocket")
             .short("s")
             .long("websocket")
             .value_name("WEBSOCKET")
             .help("Game websocket url")
             .takes_value(true))
        .arg(Arg::with_name("count")
             .short("n")
             .long("count")
             .value_name("COUNT")
             .help("Number of bots to start")
             .takes_value(true))
        ;
    let matches = app.get_matches();

    let str_command = matches.value_of("command").unwrap_or("play"); 
    let joincode = matches.value_of("joincode").unwrap_or(""); 
    let str_websocket = matches.value_of("websocket").unwrap_or("ws://127.0.0.1:8001/ws/"); 
    let count = matches.value_of("count").and_then(|str_count| str_count.parse::<usize>().ok()).unwrap_or(1); 

    if let Some(str_socket) = matches.value_of("socket"){
        let socket_file = std::path::Path::new(str_socket);
        if socket_file.exists() {
            println!("Removing dangling socket file {:?} ", socket_file);
            std::fs::remove_file(&socket_file).unwrap();
        }
        if let Ok(listener) = UnixListener::bind(socket_file) {
            println!("Listening to socket {}", str_socket);
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let stream = BufReader::new(stream);
                        let url_websocket = format!("{}/new_new", str_websocket);
                        let mut code_count = 0;
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
                                    let mut bot = player::SocketPlayer::new(tarot_socket, code.to_string(), nickname);
                                    bot.play();
                                    println!("a bot finished");
                                });
                            }
                            code_count = code_count + 1;
                        }
                        println!("Shuting down bot listener");
                        drop(listener);
                        if socket_file.exists() {
                            std::fs::remove_file(&socket_file).unwrap();
                        }
                        break
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                        break;
                    }
                }
            }
        } else {
            println!("couldn't connect to socket {}", str_socket);
        }

    } else {
        match str_command {
            "find_decks" => explorer::find_decks(),
            "play" => player_factory::play(joincode, str_websocket, count),
            _ => println!("Nothing to do")
        }
    } 
}
