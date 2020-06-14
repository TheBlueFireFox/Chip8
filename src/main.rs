use lib::{
    resources::{
        RomArchives
    },
    chip8::PROGRAM_COUNTER
};

fn main() {
    let mut rom = RomArchives::new();
    let mut files = rom.file_names();
    
    files.sort();

    for file in files {
        let rom = rom.get_file_data(&file).unwrap();
        println!("name {} len {}", file, rom.data.len());

        for i in (0..rom.data.len()).step_by(6) {
            let n = (i+5).min(rom.data.len()-1);
            print!("{:#06X} - {:#06X} : ", i + PROGRAM_COUNTER, n + PROGRAM_COUNTER);
            
            for j in i..n {
                let opcode = u16::from_be_bytes(
                    [rom.data[j], rom.data[j+1]]
                );
                print!("{:#06X} ", opcode);
            }
            println!();
        }
    }

}
