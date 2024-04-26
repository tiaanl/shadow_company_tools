#![allow(dead_code)]

use clap::Parser;
use shadow_company_tools::config::{Config, ConfigReader};
use shadow_company_tools_derive::Config;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Opts {
    /// Path to the image_defs.txt configuration file.
    config_file: PathBuf,
}

// SPRITEFRAME <x1> <y1> <x2> <y2>
#[derive(Config, Debug, Default)]
struct SpriteFrame {
    #[param(0)]
    x1: i32,
    #[param(1)]
    y1: i32,
    #[param(2)]
    x2: i32,
    #[param(3)]
    y2: i32,
}

// SPRITEFRAME_XRUN <X1> <Y1> <DX> <DY> <NUM_FRAMES>
#[derive(Config, Debug, Default)]
struct SpriteFrameXRun {
    #[param(0)]
    x1: i32,
    #[param(1)]
    y1: i32,
    #[param(2)]
    x2: i32,
    #[param(3)]
    y2: i32,
    #[param(4)]
    num_frames: i32,
}

// SPRITEFRAME_DXRUN <X1> <Y1> <SEP_DX> <DY> <NUM_FRAME> <DX * NUM_FRAMES>
#[derive(Config, Debug, Default)]
struct SpriteFrameDxRun {
    #[param(0)]
    x1: i32,
    #[param(1)]
    y1: i32,
    #[param(2)]
    sep_dx: i32,
    #[param(3)]
    dy: i32,
    #[param(4)]
    num_frame: i32,
    #[param(5)]
    dx: i32,
}

// SPRITE3D <NAME> <TEXTURENAME> <TXTR_WIDTH> <TXTR_HEIGHT> [<ALPHA>] [<Color Key Enable>] [ <Rl> <Gl> <Bl> <Rh> <Gh> Bh> ]
#[derive(Config, Debug, Default)]
struct Sprite3d {
    #[param(0)]
    name: String,
    #[param(1)]
    texture_name: String,
    #[param(2)]
    texture_width: i32,
    #[param(3)]
    texture_height: i32,
    #[param(4)]
    alpha: i32,

    #[config(key = "SPRITEFRAME")]
    sprite_frames: Vec<SpriteFrame>,
}

#[derive(Config, Debug, Default)]
struct Image {
    #[param(0)]
    name: String,
    #[param(1)]
    filename: String,
    #[param(2)]
    vid_mem: i32,
}

#[derive(Config, Debug, Default)]
struct FrameDescriptor {
    #[param(0)]
    num_images: i32,
    #[param(1)]
    num_frames: i32,
    #[param(2)]
    frame_rate: i32,
}

#[derive(Config, Debug, Default)]
struct FrameOrder {
    #[param(0)]
    order: Vec<i32>,
}

#[derive(Config, Debug, Default)]
struct AnimSprite {
    #[param(0)]
    name: String,
    #[param(1)]
    texture_name: String,
    #[param(2)]
    width: i32,
    #[param(3)]
    height: i32,

    #[config("FRAMEDESCRIPTOR")]
    frame_descriptor: FrameDescriptor,
    #[config("FRAMEORDER")]
    frame_orders: Vec<FrameOrder>,
    #[config("SPRITEFRAME")]
    sprite_frames: Vec<SpriteFrame>,
    #[config("SPRITEFRAME_XRUN")]
    sprite_frame_xruns: Vec<SpriteFrameXRun>,
    #[config("SPRITEFRAME_DXRUN")]
    sprite_frame_dxruns: Vec<SpriteFrameDxRun>,
}

#[derive(Config, Debug, Default)]
struct ImageDefs {
    #[config("IMAGE")]
    images: Vec<Image>,
    #[config("SPRITE3D", end = "ENDDEF")]
    sprite_3ds: Vec<Sprite3d>,
    #[config("ANIMSPRITE3D", end = "ENDDEF")]
    anim_sprite_3ds: Vec<AnimSprite>,
    #[config("ANIMSPRITE", end = "ENDDEF")]
    anim_sprites: Vec<AnimSprite>,
}

fn main() {
    let fm = shadow_company_tools::fm::FileManager::new("C:\\Games\\shadow_company\\Data");

    let file = match fm.open_file("config\\image_defs.txt") {
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
