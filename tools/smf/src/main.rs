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
    /// Path to the .smf file you want to operate on.
    path: PathBuf,
    /// Print out mesh vertices, index and faces.
    #[arg(long)]
    print_mesh_details: bool,
}

fn main() {
    let opts = Opts::parse();

    let mut c = Cursor::new(std::fs::read(opts.path).unwrap());

    let scene = smf::Scene::read(&mut c).expect("Could not read SMF model file.");

    println!("Model: {}, scale: {:?}", scene.name, scene.scale);
    scene.nodes.iter().for_each(|node| {
        println!(
            "  Node({}): {} ({}), position: {}, rotation: {}",
            if node.bone_index == 0xFFFFFFFF {
                "?".to_string()
            } else {
                format!("{}", node.bone_index)
            },
            node.name,
            node.parent_name,
            node.position,
            node.rotation
        );
        node.meshes.iter().for_each(|m| {
            println!(
                "    Mesh: {}, texture: {}, vertices: {}",
                m.name,
                m.texture_name,
                m.vertices.len()
            );

            if opts.print_mesh_details {
                m.vertices.iter().enumerate().for_each(|(i, v)| {
                    println!(
                        "      vertex {i}: {:9.3} {:9.3} {:9.3}",
                        v.position.x, v.position.y, v.position.z,
                    );
                });
                m.vertices.iter().enumerate().for_each(|(i, v)| {
                    println!(
                        "      normal {i}: {:9.3} {:9.3} {:9.3}",
                        v.normal.x, v.normal.y, v.normal.z,
                    );
                });
                m.vertices.iter().enumerate().for_each(|(i, v)| {
                    println!("      uv {i}: {:9.3} {:9.3}", v.tex_coord.x, v.tex_coord.y);
                });
                m.faces.iter().enumerate().for_each(|(i, f)| {
                    println!(
                        "      index {i}: {:5} {:5} {:5}",
                        f.indices[0], f.indices[1], f.indices[2]
                    );
                });
            }
        });
        node.collision_boxes.iter().for_each(|c| {
            println!("    Collision box: {} {} {}", c.max, c.min, c.u0);
        });
    });
}
