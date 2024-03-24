use clap::Parser;
use shadow_company_tools::common::hash;

#[derive(Parser)]
struct Opts {
    inputs: Vec<String>,
}

fn main() {
    let opts = Opts::parse();

    opts.inputs
        .iter()
        .map(|i| (i, hash(i.as_bytes())))
        .for_each(|(input, hash)| println!("{:08X}  {}", hash, input));
}
