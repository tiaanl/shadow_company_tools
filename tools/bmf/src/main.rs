use std::path::PathBuf;

use clap::Parser;

fn motion_state_name(state: u32) -> &'static str {
    match state {
        1 => "stand",
        2 => "crouch",
        3 => "prone",
        4 => "on_back",
        5 => "sit",
        6 => "scuba",
        _ => "unknown",
    }
}

#[derive(Parser)]
struct Opts {
    /// Path to a .bmf file to inspect.
    path: PathBuf,
    /// Print out each key frame and its data.
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,
}

fn main() {
    let opts = Opts::parse();

    if !opts.path.exists() {
        eprintln!("Path does not exist: {}", opts.path.display());
        std::process::exit(1)
    }

    let paths: Vec<PathBuf> = if opts.path.is_dir() {
        walkdir::WalkDir::new(&opts.path)
            .into_iter()
            .filter_map(|result| result.ok().map(|entry| entry.path().to_path_buf()))
            .filter(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("bmf"))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        vec![opts.path]
    };

    for path in paths {
        let mut file = std::fs::File::open(&path).expect("Could not open file");
        let motion =
            shadow_company_tools::bmf::Motion::read(&mut file).expect("Could not read motion.");

        println!("Motion: {} (0x{:08X})", motion.name, motion.hash);
        let from_state_name = motion_state_name(motion.from_state);
        let to_state_name = motion_state_name(motion.to_state);
        println!(
            "  flags: {:?}, key_frame_count: {}, last_frame: {}, max_bones_per_frame: {}, ticks_per_frame: {}",
            motion.flags,
            motion.key_frame_count,
            motion.last_frame,
            motion.max_bones_per_frame,
            motion.ticks_per_frame,
        );
        println!(
            "  state_transition: {} ({}) -> {} ({})",
            motion.from_state, from_state_name, motion.to_state, to_state_name
        );

        if !opts.verbose {
            continue;
        }

        motion.key_frames.iter().for_each(|kf| {
            println!("  Time: {:3}", kf.frame);
            println!(
                "    lve: ({:8.03}, {:8.03}, {:8.03})  bone_count: {:3}  reserved: ({}, {})",
                kf.lve.x, kf.lve.y, kf.lve.z, kf.bone_count, kf.reserved_0, kf.reserved_1
            );

            kf.bones.iter().for_each(|b| {
                print!("    {:3}: ", b.bone_id,);

                print!("position: ");
                if let Some(position) = b.position {
                    print!(
                        "{:7.02}, {:7.02}, {:7.02},",
                        position.x, position.y, position.z,
                    );
                } else {
                    print!("      -,       -,       -,");
                }

                print!(" rotation: ");
                if let Some(rotation) = b.rotation {
                    print!(
                        "{:7.02}, {:7.02}, {:7.02}, {:7.02}",
                        rotation.x, rotation.y, rotation.z, rotation.w
                    );
                } else {
                    print!("      -,       -,       -,       -");
                }

                println!();
            });
        });
    }
}
