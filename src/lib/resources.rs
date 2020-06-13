use std::{
    self,
    io::{
        Cursor
    }
};
use zip::{
    read::ZipArchive,
    result::ZipResult
};


const ROM_ARCHIVE : &'static [u8] = std::include_bytes!("resources/c8games.zip");

pub struct Rom;

impl Rom {
    pub fn new() -> Self {
        Rom {}
    }

    /// Will retuan all the rom names availale to be chosen
    pub fn file_names(&self) -> ZipResult<Vec<String>> {
        let reader = Cursor::new(ROM_ARCHIVE);

        let archive_reader = ZipArchive::new(reader)?; 
        let mut data = Vec::new();

        for file in archive_reader.file_names() {
            data.push(file.to_string());
        }

        Ok(data)
    }

    //pub fn get_file_data(name : String) -> &[u8] {
        //let reader = Cursor::new(ROM_ARCHIVE);
        //let archive_reader = ZipArchive::new(reader)?; 
    //}
}