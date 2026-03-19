use std::{env, fs};

use webtarot_protocol::TarotGameState;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        let file_path = args[i].as_str();
        let error = format!("Could not read file {file_path}...");
        let json_str = fs::read_to_string(file_path).expect(&error);
        let game: TarotGameState = serde_json::from_str(&json_str).expect("Error parsing JSON");
        println!("{game}");
        i += 1;
    }
}
