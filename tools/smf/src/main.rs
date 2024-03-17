#![allow(dead_code)]
#![allow(unused_imports)]

use base64::Engine;
use clap::Parser;
use json::material::PbrMetallicRoughness;
use shadow_company_tools::{bmf, smf};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;

use gltf_json as json;
use json::validation::Checked::Valid;
use json::validation::USize64;

#[derive(Debug, Parser)]
struct Opts {
    /// path to the .smf file you want to operate on
    path: PathBuf,
}

fn main() {
    let opts = Opts::parse();

    let mut c = Cursor::new(std::fs::read(opts.path).unwrap());

    let scene = smf::Scene::read(&mut c).expect("Could not read SMF model file.");

    println!("Model: {}, scale: {:?}", scene.name, scene.scale);
    scene.nodes.iter().for_each(|model| {
        println!(
            "  Node: {} ({}), position: {:?}, rotation: {:?}",
            model.name, model.parent_name, model.position, model.rotation
        );
        model.meshes.iter().for_each(|m| {
            println!(
                "    Mesh: {}, texture: {}, vertices: {}",
                m.name,
                m.texture_name,
                m.vertices.len()
            );

            m.vertices.iter().enumerate().for_each(|(i, v)| {
                println!(
                    "      vertex {i}: {:9.3} {:9.3} {:9.3}",
                    v.position.0, v.position.1, v.position.2,
                );
            });
            m.vertices.iter().enumerate().for_each(|(i, v)| {
                println!(
                    "      normal {i}: {:9.3} {:9.3} {:9.3}",
                    v.normal.0, v.normal.1, v.normal.2,
                );
            });
            m.faces.iter().enumerate().for_each(|(i, f)| {
                println!(
                    "      index {i}: {:5} {:5} {:5}",
                    f.indices[0], f.indices[1], f.indices[2]
                );
            });
        });
    });
}
