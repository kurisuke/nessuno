use serde::Deserialize;
use serde_xml_rs::from_str;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Datafile {
    game: Vec<Game>,
}

#[derive(Deserialize)]
struct Game {
    name: String,
    rom: Vec<Rom>,
}

#[derive(Deserialize)]
struct Rom {
    sha1: String,
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let xml = std::fs::read_to_string(&args[1]).unwrap();

    let datafile: Datafile = from_str(&xml).unwrap();

    let mut rom_db = HashMap::new();

    for game in datafile.game {
        for rom in game.rom {
            rom_db.insert(rom.sha1.clone(), game.name.clone());
        }
    }

    nessuno::romdb::save(&rom_db);
}
