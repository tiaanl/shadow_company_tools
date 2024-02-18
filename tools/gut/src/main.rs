use clap::Parser;
use shadow_company_tools::gut::GutFile;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Opts {
    /// path the .gut file you want to operate on
    path: PathBuf,
}

fn main() {
    let opts = Opts::parse();

    let gut_file = GutFile::load(opts.path).unwrap();
    let mut entries = gut_file.read_entries();
    entries.sort_by(|a, b| a.size.cmp(&b.size));
    for entry in entries {
        println!(
            "{} ({} bytes{})",
            entry.name,
            entry.size,
            if !entry.is_plain_text {
                ""
            } else {
                ", obfuscated"
            }
        );
    }
}

// EXTRACT ALL .gut FILES
// ----------------------

// let encoded = encode_path("AUDIO_NULL".as_bytes());

// println!("encoded: {:08X}", encoded);
// let decoded = decode_path(encoded);
// println!("encoded: {:04X}, decoded: {}", encoded, decoded);

// let extract_path = PathBuf::from("C:\\Games\\shadow_company\\extracted");

// walkdir::WalkDir::new("C:\\Games\\shadow_company")
//     .into_iter()
//     .filter_map(|e| e.ok())
//     .filter(|e| e.path().extension().unwrap_or_default() == "gut")
//     .for_each(|e| {
//         let prefix = e
//             .path()
//             .strip_prefix("C:\\Games\\shadow_company")
//             .unwrap()
//             .parent()
//             .unwrap();

//         let mut gut_file = GutFile::load(e.path()).unwrap();
//         let entries = gut_file.read_entries();

//         // println!("[{}]", gut_file.path.display());
//         for entry in entries {
//             // println!("{}: {} ({} bytes)", entry.name, entry.offset, entry.size);

//             /*
//             let c = gut_file.get_contents(&entry);
//             for ch in c.iter() {
//                 print!("{}", *ch as char);
//             }
//             println!();
//             */
//             let full_path = extract_path.join(prefix).join(&entry.name);
//             println!("{}: full_path: {}", entry.is_encrypted, full_path.display());
//             std::fs::create_dir_all(full_path.parent().unwrap()).unwrap();
//             std::fs::write(full_path, gut_file.get_contents(&entry)).unwrap();
//         }
//     });
