use clap::{Parser, Subcommand};
use shadow_company_tools::gut::GutFile;
use std::path::{Path, PathBuf};

#[derive(Parser)]
struct Opts {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List the contents of a .gut file.
    List {
        /// path the .gut file you want to operate on
        path: PathBuf,
    },
    /// Extract the contents of a .gut file.
    Extract {
        /// Path to the .gut file to extract.
        path: PathBuf,
        /// An output directory where file will be extracted to.
        out_dir: PathBuf,
    },
}

fn main() {
    let opts = Opts::parse();

    match opts.command {
        Commands::List { path } => list(path),
        Commands::Extract { path, out_dir } => extract(path, out_dir),
    }
}

fn list(path: impl AsRef<Path>) {
    let gut_file = GutFile::load(path.as_ref()).unwrap();
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

fn extract(path: impl AsRef<Path>, out_dir: impl AsRef<Path>) {
    let gut_file_paths = if path.as_ref().is_dir() {
        walkdir::WalkDir::new(path.as_ref())
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.path().extension().unwrap_or_default() == "gut")
            .map(|f| f.path().to_owned())
            .collect()
    } else {
        vec![path.as_ref().to_owned()]
    };

    for gut_file_path in gut_file_paths {
        println!("Extracting contents of {}", gut_file_path.display());
        let mut gut_file = GutFile::load(gut_file_path).unwrap();
        let entries = gut_file.read_entries();

        for entry in entries {
            let full_path = out_dir.as_ref().join(&entry.name);
            println!("  - {}", entry.name);
            std::fs::create_dir_all(full_path.parent().unwrap()).unwrap();
            std::fs::write(full_path, gut_file.get_contents(&entry)).unwrap();
        }
    }
}

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
