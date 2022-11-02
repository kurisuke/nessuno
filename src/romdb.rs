use directories::BaseDirs;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

pub type RomDb = HashMap<String, String>;

pub fn save(db: &RomDb) -> bool {
    let mut p = rom_db_dir_path();
    std::fs::create_dir_all(&p).unwrap();

    p.push("rom.db");
    let writer = BufWriter::new(File::create(&p).unwrap());
    let mut encoder = ZlibEncoder::new(writer, Compression::best());
    bincode::serialize_into(&mut encoder, db).is_ok()
}

pub fn load() -> Option<RomDb> {
    let mut p = rom_db_dir_path();
    p.push("rom.db");

    if p.is_file() {
        let reader = BufReader::new(File::open(&p).unwrap());
        let mut decoder = ZlibDecoder::new(reader);
        bincode::deserialize_from(&mut decoder).ok()
    } else {
        None
    }
}

fn rom_db_dir_path() -> PathBuf {
    // find & create save directory
    let base_dirs = BaseDirs::new().unwrap();
    let mut rom_db_dir_buf = PathBuf::new();
    rom_db_dir_buf.push(base_dirs.data_dir());
    rom_db_dir_buf.push("nessuno");
    rom_db_dir_buf.push("db");
    rom_db_dir_buf
}
