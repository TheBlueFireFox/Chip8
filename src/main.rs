use lib::{
    resources::{Rom
    },
    chip8::PROGRAM_COUNTER_BASE
};

fn main() {
    let mut rom = Rom::new();
    let files = rom.file_names();
    
    for file in files {
        let data = rom.get_file_data(&file).unwrap();
        println!("name {} len {}", file, data.len());

        for i in (0..data.len()).step_by(6) {
            let n = (i+5).min(data.len()-1);
            print!("{:#06X} - {:#06X} : ", i + PROGRAM_COUNTER_BASE, n + PROGRAM_COUNTER_BASE);
            
            for j in i..n {
                let opcode = u16::from_be_bytes(
                    [data[j], data[j+1]]
                );
                print!("{:#06X} ", opcode);
            }
            println!();
        }
    }

}
