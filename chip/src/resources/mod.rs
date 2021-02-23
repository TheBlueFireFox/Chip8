use std::{
    self,
    io::{prelude::*, Cursor},
};
use zip::{read::ZipArchive, result::ZipResult};

/// Contains all the available roms needed for running the games
/// in a ZIP archive.
const ROM_ARCHIVE: &'static [u8] = std::include_bytes!("c8games.zip");

/// Represents an archive of roms
/// it contains all kind of information about the information of the archives
pub struct RomArchives<'a> {
    archive: ZipArchive<Cursor<&'a [u8]>>,
}

impl RomArchives<'_> {
    /// Will generate a new rom archive object based of the given rom archive
    pub fn new() -> Self {
        RomArchives {
            // can be directly unwrapped, as the rom archive has already been manually checked
            archive: ZipArchive::new(Cursor::new(ROM_ARCHIVE)).unwrap(),
        }
    }

    /// Will return all the rom names available to be chosen
    pub fn file_names(&self) -> Vec<&'_ str> {
        self.archive.file_names().collect()
    }

    // Will decompress the information from the zip archive
    pub fn get_file_data(&mut self, name: &str) -> ZipResult<Rom> {
        let mut file = self.archive.by_name(name)?;
        // there might be a case where there is an uneven amount of
        // data entries adding one for simplicty.
        let size = (file.size() + file.size() % 2) as usize;
        let mut data = vec![0; size].into_boxed_slice();
        // this result can be ignored as the included archive
        // will definitely contain data for if the file is included
        file.read(&mut data)?;
        Ok(Rom::new(name, data))
    }
}

#[derive(Clone)]
/// Represents a single rom with it's information
pub struct Rom {
    /// The rom name
    name: String,
    /// The decompressed content data of the zip file
    /// stored as a u8 slice on the heap
    /// uses a box for simple execution
    data: Box<[u8]>,
}

impl Rom {
    /// Will generate a new rom based of the given data
    fn new(name: &str, data: Box<[u8]>) -> Self {
        Rom {
            name: name.to_string(),
            data,
        }
    }

    /// Will return a slice internal values of the given data
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    /// Will return the name of the rom.
    pub fn get_name(&self) -> &str {
        &self.name
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

    const ROM_NAMES: [&str; 24] = [
        "15PUZZLE", "BLINKY", "BLITZ", "BRIX", "CONNECT4", "GUESS", "HIDDEN", "IBMLOGO", "INVADERS", "KALEID",
        "MAZE", "MERLIN", "MISSILE", "PONG", "PONG2", "PUZZLE", "SYZYGY", "TANK", "TETRIS",
        "TICTAC", "UFO", "VBRIX", "VERS", "WIPEOFF", 
    ];

    #[test]
    fn test_rom_extract() {
        let mut ra = RomArchives::new();
        let name = ROM_NAMES[0];
        let rom = ra.get_file_data(name).unwrap();
        let data = rom.get_data();

        for i in (0..data.len()).step_by(2) {
            let output = build_opcode(data, i);
            assert!(output.is_ok());
            let opcode: Opcode = output.unwrap();
            assert_eq!(RAW_ROM_DATA[i / 2], opcode);
        }
    }

    #[test]
    fn test_file_names() {
        let ra = RomArchives::new();
        let mut files = ra.file_names();
        files.sort();

        assert_eq!(ROM_NAMES.len(), files.len());

        assert_eq!(&ROM_NAMES, &files[..]);
    }
}
