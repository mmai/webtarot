use clap::{Arg, App};

mod explorer;
mod simulator;
mod socket_listener;
mod player;
mod player_factory;

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
    let str_websocket = matches.value_of("websocket").unwrap_or("ws://127.0.0.1:8001"); 
    let count = matches.value_of("count").and_then(|str_count| str_count.parse::<usize>().ok()).unwrap_or(1); 

    if let Some(str_socket) = matches.value_of("socket"){
        socket_listener::start(str_socket, str_websocket);
    } else {
        match str_command {
            "find_decks" => explorer::find_decks(),
            "simulate" => simulator::simulate(),
            "play" => player_factory::play(joincode, str_websocket, count),
            _ => println!("Nothing to do")
        }
    } 
}
