use serde::Deserialize;
use serde_xml_rs::from_str;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Datafile {
    game: Vec<Game>,
}

#[derive(Deserialize)]
struct Game {
    #[serde(rename = "@name")]
    name: String,
    rom: Vec<Rom>,
}

#[derive(Deserialize)]
struct Rom {
    #[serde(rename = "@sha1")]
    sha1: String,
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let xml = std::fs::read_to_string(&args[1]).unwrap();

    let datafile: Datafile = from_str(&xml).unwrap();

    let mut rom_db = HashMap::new();

    let mut num_games = 0;
    let mut num_roms = 0;

    for game in datafile.game {
        num_games += 1;
        for rom in game.rom {
            num_roms += 1;
            rom_db.insert(rom.sha1.clone(), game.name.clone());
        }
    }

    println!("Found games: {num_games}");
    println!("Found roms: {num_roms}");

    nessuno::romdb::save(&rom_db);
}
