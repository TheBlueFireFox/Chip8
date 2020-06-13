use std::{
    self,
    io::{
        prelude::*,
        Cursor
    }
};
use zip::{
    read::ZipArchive,
    result::ZipResult
};


const ROM_ARCHIVE : &'static [u8] = std::include_bytes!("resources/c8games.zip");

pub struct Rom<'a> {
    archive : ZipArchive<Cursor<&'a [u8]>>
}

impl Rom<'_> {
    pub fn new() -> Self {
        Rom {
            archive : ZipArchive::new(Cursor::new(ROM_ARCHIVE)).unwrap()
        }
    }

    /// Will retuan all the rom names availale to be chosen
    pub fn file_names(&self) -> Vec<String> {
        
        let mut data = Vec::new();

        for file in self.archive.file_names() {
            data.push(file.to_string());
        }

        data
    }

    // Will decompress the information from the zip archive
    pub fn get_file_data(&mut self, name : &str) -> ZipResult<Vec<u8>> {
        let mut file = self.archive.by_name(name)?;

        let mut data = vec![0; file.size() as usize];

        let _ = file.read(&mut data);
        Ok(data)
    }
}