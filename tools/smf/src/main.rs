use byteorder::{LittleEndian, ReadBytesExt};
use clap::Parser;
use shadow_company_tools::smf::Scene;
use std::io::Cursor;
use std::{io::BufRead, path::PathBuf};

#[derive(Debug, Parser)]
struct Opts {
    /// path to the .smf file you want to operate on
    path: PathBuf,
}

fn main() {
    let opts = Opts::parse();

    let mut c = Cursor::new(std::fs::read(&opts.path).unwrap());

    // read header.
    let mut header = vec![];
    for _ in 0..8 {
        c.read_until('\n' as u8, &mut header).unwrap();
    }

    let _u1 = c.read_u32::<LittleEndian>().unwrap();
    // println!("unknown: {:08X}", _u1);
    let _u2 = c.read_u32::<LittleEndian>().unwrap();
    // println!("unknown: {:08X}", _u2);

    let scene = Scene::read(&mut c);

    println!("{:?}", scene);
}
