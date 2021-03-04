use chip::{
    chip8::ChipSet,
    resources::{Rom, RomArchives},
    timer::{NoCallback, Worker},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const ROM_NAME: &'static str = "15PUZZLE";

lazy_static::lazy_static! {
    /// preloading this as it get's called multiple times per unit
    static ref BASE_ROM : Rom = get_rom(ROM_NAME);
}

fn get_rom(s: &str) -> Rom {
    RomArchives::new()
        .get_file_data(s)
        .expect("A panic happend during extraction of the Rom archive.")
}

fn get_base() -> Rom {
    BASE_ROM.clone()
}

/// will setup the default configured chip
fn get_default_chip() -> ChipSet<Worker, NoCallback> {
    let rom = get_base();
    setup_chip(rom)
}

fn setup_chip(rom: Rom) -> ChipSet<Worker, NoCallback> {
    ChipSet::new(rom)
}

pub fn print_bench(c: &mut Criterion) {
    let chip = get_default_chip();
    c.bench_function("print_bench", |b| {
        b.iter(|| {
            let _ = format!("{}", chip);
        });
    });
}

criterion_group!(benches, print_bench);
criterion_main!(benches);
