use clap::Parser;
use shadow_company_tools::smf::Scene;
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Opts {
    /// path to the .smf file you want to operate on
    path: PathBuf,
}

fn main() {
    let opts = Opts::parse();

    let mut c = Cursor::new(std::fs::read(&opts.path).unwrap());

    shadow_company_tools::common::skip_sinister_header(&mut c).unwrap();

    let scene = Scene::read(&mut c);

    println!("{:?}", scene);
}
