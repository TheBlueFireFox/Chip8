use std::{
    self,
    io::{prelude::*, Cursor},
    ops
};
use zip::{read::ZipArchive, result::ZipResult};

#[cfg(target_os="windows")]
/// Contains all the available roms needed for running the games
/// in a ZIP archive (the path used works for windows)
const ROM_ARCHIVE: &'static [u8] = std::include_bytes!("resources\\c8games.zip");

#[cfg(not(target_os="windows"))]
/// Contains all the available roms needed for running the games
/// in a ZIP archive (the path used works for unix)
const ROM_ARCHIVE: &'static [u8] = std::include_bytes!("resources/c8games.zip");

/// Represents an archive of roms
/// it contains all kind of information about the information of the archives
pub struct RomArchives<'a> {
    archive: ZipArchive<Cursor<&'a [u8]>>,
}

impl RomArchives<'_> {
    /// Will generate a new rom archive objecz based of the given rom archive
    pub fn new() -> Self {
        RomArchives {
            archive: ZipArchive::new(Cursor::new(ROM_ARCHIVE)).unwrap(),
        }
    }

    /// Will retuan all the rom names available to be chosen
    pub fn file_names(&self) -> Vec<String> {
        let mut data = Vec::new();

        for file in self.archive.file_names() {
            data.push(file.to_string());
        }
        data
    }

    // Will decompress the information from the zip archive
    pub fn get_file_data(&mut self, name: &str) -> ZipResult<Rom> {
        let mut file = self.archive.by_name(name)?;
        let mut data = vec![0; file.size() as usize];
        // this result can be ignored as the included archive
        // will definitely contain data for if the file is included
        let _ = file.read(&mut data);
        if data.len() % 2 == 1 {
            data.push(0x0);
        }
        Ok(Rom::new(data))
    }
}

#[derive(Clone)]
/// Represents a single rom with it's information
pub struct Rom {
    /// The decompressed content data of the zip file
    /// stored as a u8 slice on the heap
    /// uses a box for simple execution 
    data: Box<[u8]>,
}

impl Rom {
    /// Will generate a new rom based of the given data
    fn new(data: Vec<u8>) -> Self {
        Rom { data : data.into_boxed_slice() }
    }

    /// Will return a slice internal values of the given data
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_file_names() {
        let mut data = 
        "15PUZZLE
        BLINKY
        BLITZ
        BRIX
        CONNECT4
        GUESS
        HIDDEN
        INVADERS
        KALEID
        MAZE
        MERLIN
        MISSILE
        PONG
        PONG2
        PUZZLE
        SYZYGY
        TANK
        TETRIS
        TICTAC
        UFO
        VBRIX
        VERS
        WIPEOFF".split("\n")
            .map(|x| x.trim().to_string())
            .collect::<Vec<_>>();
        data.sort();

        let ra = RomArchives::new();
        let mut files = ra.file_names();
        files.sort();
        assert_eq!(data.len(), files.len());

        for i in 0..(data.len()) {
            assert_eq!(data[i], files[i]);
        }
    }

}
