use chip8_lib::{
    resources::RomArchives,
    chip8::PROGRAM_COUNTER
};

fn main() {
    let mut rom = RomArchives::new();
    let mut files = rom.file_names();
    
    files.sort();

    for file in &files[..1] {
        let rom = rom.get_file_data(&file).unwrap();
        let data = rom.get_data();
        println!("name {} len {}", file, data.len());

        for i in (0..data.len()).step_by(6) {
            let n = (i+5).min(data.len()-1);
            print!("{:#06X} - {:#06X} : ", i + PROGRAM_COUNTER, n + PROGRAM_COUNTER);
            
            for j in i..n {
                let opcode = u16::from_be_bytes([data[j], data[j+1]]);
                print!("{:#06X} ", opcode);
            }
            println!();
        }
    }

}
