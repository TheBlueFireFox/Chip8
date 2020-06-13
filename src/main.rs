use lib::resources::Rom;

fn main() {
    let mut rom = Rom::new();
    println!("{}", rom.file_names().join("\n"));
    let data = rom.get_file_data("PONG2").unwrap();



    println!("len {}", data.len());
    for i in data {
        print!("{:#04} ", i);
    }
    println!();   
}
