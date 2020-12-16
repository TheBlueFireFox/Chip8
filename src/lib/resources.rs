use std::{
    self,
    io::{prelude::*, Cursor},
};
use zip::{read::ZipArchive, result::ZipResult};

#[cfg(target_os = "windows")]
/// Contains all the available roms needed for running the games
/// in a ZIP archive (the path used works for windows)
const ROM_ARCHIVE: &'static [u8] = std::include_bytes!("resources\\c8games.zip");

#[cfg(not(target_os = "windows"))]
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
            // can be directly unwraped, as the rom archive has already been manually checked
            archive: ZipArchive::new(Cursor::new(ROM_ARCHIVE)).unwrap(),
        }
    }

    /// Will retuan all the rom names available to be chosen
    pub fn file_names(&self) -> Vec<&'_ str> {
        let mut data = Vec::new();

        for file in self.archive.file_names() {
            data.push(file);
        }
        data
    }

    // Will decompress the information from the zip archive
    pub fn get_file_data(&mut self, name: &str) -> ZipResult<Rom> {
        let mut file = self.archive.by_name(name)?;
        let mut size = file.size() as usize;
        if size % 2 == 1 {
            size += 1;
        }
        let mut data = vec![0; size].into_boxed_slice();
        // this result can be ignored as the included archive
        // will definitely contain data for if the file is included
        file.read(&mut data)?;
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
    fn new(data: Box<[u8]>) -> Self {
        Rom { data }
    }

    /// Will return a slice internal values of the given data
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::RomArchives;
    use crate::opcode::{build_opcode, Opcode};
    const RAW_ROM_DATA: [Opcode; 192] = [
        0x00E0, 0x6C00, 0x4C00, 0x6E0F, 0xA203, 0x6020, 0xF055, 0x00E0, 0x22BE, 0x2276, 0x228E,
        0x225E, 0x2246, 0x1210, 0x6100, 0x6217, 0x6304, 0x4110, 0x00EE, 0xA2E8, 0xF11E, 0xF065,
        0x4000, 0x1234, 0xF029, 0xD235, 0x7101, 0x7205, 0x6403, 0x8412, 0x3400, 0x1222, 0x6217,
        0x7306, 0x1222, 0x6403, 0x84E2, 0x6503, 0x85D2, 0x9450, 0x00EE, 0x4403, 0x00EE, 0x6401,
        0x84E4, 0x22A6, 0x1246, 0x6403, 0x84E2, 0x6503, 0x85D2, 0x9450, 0x00EE, 0x4400, 0x00EE,
        0x64FF, 0x84E4, 0x22A6, 0x125E, 0x640C, 0x84E2, 0x650C, 0x85D2, 0x9450, 0x00EE, 0x4400,
        0x00EE, 0x64FC, 0x84E4, 0x22A6, 0x1276, 0x640C, 0x84E2, 0x650C, 0x85D2, 0x9450, 0x00EE,
        0x440C, 0x00EE, 0x6404, 0x84E4, 0x22A6, 0x128E, 0xA2E8, 0xF41E, 0xF065, 0xA2E8, 0xFE1E,
        0xF055, 0x6000, 0xA2E8, 0xF41E, 0xF055, 0x8E40, 0x00EE, 0x3C00, 0x12D2, 0x221C, 0x22D8,
        0x221C, 0xA2F8, 0xFD1E, 0xF065, 0x8D00, 0x00EE, 0x7CFF, 0xCD0F, 0x00EE, 0x7D01, 0x600F,
        0x8D02, 0xED9E, 0x12D8, 0xEDA1, 0x12E2, 0x00EE, 0x0102, 0x0304, 0x0506, 0x0708, 0x090A,
        0x0B0C, 0x0D0E, 0x0F00, 0x0D00, 0x0102, 0x0405, 0x0608, 0x090A, 0x0C0E, 0x0307, 0x0B0F,
        0x84E4, 0x22A6, 0x1276, 0x640C, 0x84E2, 0x650C, 0x85D2, 0x9450, 0x00EE, 0x440C, 0x00EE,
        0x6404, 0x84E4, 0x22A6, 0x128E, 0xA2E8, 0xF41E, 0xF065, 0xA2E8, 0xFE1E, 0xF055, 0x6000,
        0xA2E8, 0xF41E, 0xF055, 0x8E40, 0x00EE, 0x3C00, 0x12D2, 0x221C, 0x22D8, 0x221C, 0xA2F8,
        0xFD1E, 0xF065, 0x8D00, 0x00EE, 0x7CFF, 0xCD0F, 0x00EE, 0x7D01, 0x600F, 0x8D02, 0xED9E,
        0x12D8, 0xEDA1, 0x12E2, 0x00EE, 0x0102, 0x0304, 0x0506, 0x0708, 0x090A, 0x0B0C, 0x0D0E,
        0x0F00, 0x0D00, 0x0102, 0x0405, 0x0608,
    ];

    const ROM_NAMES: [&str; 23] = [
        "15PUZZLE", "BLINKY", "BLITZ", "BRIX", "CONNECT4", "GUESS", "HIDDEN", "INVADERS", "KALEID",
        "MAZE", "MERLIN", "MISSILE", "PONG", "PONG2", "PUZZLE", "SYZYGY", "TANK", "TETRIS",
        "TICTAC", "UFO", "VBRIX", "VERS", "WIPEOFF",
    ];

    #[test]
    fn test_rom_extract() {
        let mut ra = RomArchives::new();
        let name = "15PUZZLE";
        let rom = ra.get_file_data(name).unwrap();
        let data = rom.get_data();

        for i in (0..data.len()).step_by(2) {
            let opcode: Opcode = build_opcode(data, i);
            assert_eq!(RAW_ROM_DATA[i / 2], opcode);
        }
    }

    #[test]
    fn test_file_names() {
        let ra = RomArchives::new();
        let mut files = ra.file_names();
        files.sort();
        assert_eq!(ROM_NAMES.len(), files.len());

        for i in 0..(files.len()) {
            assert_eq!(ROM_NAMES[i], files[i]);
        }
    }
}
