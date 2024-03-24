use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
struct Opts {
    /// Path to a .bmf file to inspect.
    path: PathBuf,
}

fn main() {
    let opts = Opts::parse();

    let mut file = std::fs::File::open(opts.path).expect("Could not open file");
    let motion =
        shadow_company_tools::bmf::Motion::read(&mut file).expect("Could not read motion.");

    println!("Motion: {}", motion.name);
    motion.key_frames.iter().for_each(|kf| {
        println!("  Time: {:3}", kf.time);
        kf.bones.iter().for_each(|b| {
            println!(
                "    {:3}: rotation: {:9?} position: {:9?}",
                b.bone_index, b.rotation, b.position
            );
        });
    });
}
