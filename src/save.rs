use crate::system::System;
use directories::BaseDirs;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub struct SaveState {
    pub save_file: String,
}

impl SaveState {
    pub fn new(rom_filename: &str) -> SaveState {
        // find & create save directory
        let base_dirs = BaseDirs::new().unwrap();
        let mut save_file_buf = PathBuf::new();
        save_file_buf.push(base_dirs.data_dir());
        save_file_buf.push("nessuno");
        save_file_buf.push("saves");

        std::fs::create_dir_all(&save_file_buf).unwrap();

        // get save file name
        let rom_file_path = Path::new(rom_filename);
        let rom_file_stem = rom_file_path.file_stem().unwrap();
        save_file_buf.push(rom_file_stem);
        save_file_buf.set_extension("sav");

        SaveState {
            save_file: String::from(save_file_buf.to_str().unwrap()),
        }
    }

    pub fn load(&self) -> Option<System> {
        let save_file_path = Path::new(&self.save_file);
        if save_file_path.is_file() {
            let reader = BufReader::new(File::open(&self.save_file).unwrap());
            let mut decoder = ZlibDecoder::new(reader);
            bincode::deserialize_from(&mut decoder).ok()
        } else {
            None
        }
    }

    pub fn save(&self, system: &System) -> bool {
        let save_file_path = Path::new(&self.save_file);
        let writer = BufWriter::new(File::create(&save_file_path).unwrap());
        let mut encoder = ZlibEncoder::new(writer, Compression::best());
        bincode::serialize_into(&mut encoder, &system).is_ok()
    }
}
