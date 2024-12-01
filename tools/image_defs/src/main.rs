#![allow(dead_code)]

use clap::Parser;
use shadow_company_tools::config::{Config, ConfigReader};
use shadow_company_tools_configs::ImageDefs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Opts {
    /// Path to the image_defs.txt configuration file.
    config_file: PathBuf,
}

fn main() {
    let fm = shadow_company_tools::data_dir::DataDir::new("C:\\Games\\shadow_company\\Data");

    let file = match fm.open("config\\image_defs.txt") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    };

    let mut reader = ConfigReader::new(file).expect("failed to read a line from the config");
    let image_defs = ImageDefs::from_config(&mut reader).expect("failed to read image_defs");

    println!("Images:");
    image_defs.images.iter().for_each(|image| {
        println!("  - {}, {}", image.name, image.filename);
    });
    println!();

    println!("3D sprites:");
    image_defs.sprite_3ds.iter().for_each(|sprite| {
        println!("  - {}, {}", sprite.name, sprite.texture_name);
    });
    println!();

    println!("Animation sprites:");
    image_defs.anim_sprites.iter().for_each(|sprite| {
        println!(
            "  - {}, {}, {} frames",
            sprite.name, sprite.texture_name, sprite.frame_descriptor.num_frames
        );
    });
    println!();

    println!("3D animation sprites:");
    image_defs.anim_sprite_3ds.iter().for_each(|sprite| {
        println!(
            "  - {}, {}, {} frames",
            sprite.name, sprite.texture_name, sprite.frame_descriptor.num_frames
        );
    });
    println!();
}
