use clap::{Arg, App};

mod explorer;

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
        .arg(Arg::with_name("command")
             .short("c")
             .long("command")
             .value_name("COMMAND")
             .help("Command to execute")
             .takes_value(true))
        ;
    let matches = app.get_matches();

    let str_command = matches.value_of("command").unwrap_or("find_decks"); 

    match str_command {
        "find_decks" => explorer::find_decks(),
        _ => println!("Nothing to do")
    }
}
