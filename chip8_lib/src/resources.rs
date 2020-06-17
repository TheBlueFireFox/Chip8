use std::{
    self,
    io::{prelude::*, Cursor},
};
use zip::{read::ZipArchive, result::ZipResult};

#[cfg(target_os="windows")]
/// Contains all the available roms needed for running the games
/// in a ZIP archive (the path used works for both unix and windows)
const ROM_ARCHIVE: &'static [u8] = std::include_bytes!("resources\\c8games.zip");

#[cfg(not(target_os="windows"))]
/// Contains all the available roms needed for running the games
/// in a ZIP archive (the path used works for both unix and windows)
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
        let _ = file.read(&mut data);
        if data.len() % 2 == 1 {
            data.push(0x0);
        }
        Ok(Rom::new(data))
    }
}

/// Represents a single rom with it's information
pub struct Rom {
    data: Vec<u8>,
}

impl Rom {
    /// Will generate a new rom based of the given data
    fn new(data: Vec<u8>) -> Self {
        Rom { data }
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}
