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
    let mut file = std::fs::File::open(path.as_ref()).unwrap();
    let gut_file = GutFile::open(&mut file).unwrap();

    println!("gut_file: {:?}", gut_file);

    for entry in gut_file.entries() {
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
        let mut file = std::fs::File::open(gut_file_path).unwrap();
        let mut gut_file = GutFile::open(&mut file).unwrap();

        for entry in gut_file.entries() {
            let entry_path = entry
                .name
                .split(r"\")
                .collect::<Vec<_>>()
                .join(std::path::MAIN_SEPARATOR_STR);
            let full_path = out_dir.as_ref().join(&entry_path);
            println!("  - {}", entry_path);
            // std::fs::create_dir_all(full_path.parent().unwrap()).unwrap();
            // std::fs::write(full_path, gut_file.get_contents(&entry)).unwrap();
        }
    }
}
