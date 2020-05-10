use clap::{Arg, App};

mod game;
mod server;
mod universe;
mod utils;

pub(crate) use webgame_protocol as protocol;

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();


    let app = App::new("Webtarot")
        .version("1.0")
        .author("Henri Bourcereau <henri@bourcereau.fr>")
        .about("A online game of french tarot")
        .arg(Arg::with_name("directory")
             .short("d")
             .long("directory")
             .value_name("ROOT")
             .help("Directory path of the static files")
             .takes_value(true))
        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .value_name("PORT")
             .help("Port the server listen to")
             .takes_value(true))
        ;
    let matches = app.get_matches();

    let mut default_public_dir = get_current_dir();
    default_public_dir.push_str("/public");
    let public_dir = matches.value_of("directory").unwrap_or(&default_public_dir);
    // let pdir = std::path::PathBuf::from(public_dir);

    let str_port = matches.value_of("port").unwrap_or("8002"); 
    let port = str_port.parse::<u16>().unwrap();
    // println!("Value for directory: {}", public_dir);


    server::serve(String::from(public_dir), port).await;
}

fn get_current_dir() -> String {
    std::env::current_dir()
    .map( |cd|
          String::from(cd.as_path().to_str().unwrap())
    ).expect("Can't find current path")
}
