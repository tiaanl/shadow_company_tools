use clap::Parser;
use shadow_company_tools::smf;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Opts {
    /// Path to the .smf file you want to operate on.
    path: PathBuf,
    /// Print out mesh vertices, index and faces.
    #[arg(long)]
    print_mesh_details: bool,
}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();

    let files = if opts.path.is_dir() {
        walkdir::WalkDir::new(opts.path)
            .into_iter()
            .filter_map(|entry| {
                let Ok(entry) = entry else {
                    return None;
                };

                if entry.path().extension()? != "smf" {
                    return None;
                }

                Some(entry.into_path())
            })
            .collect()
    } else {
        vec![opts.path]
    };

    for file in files {
        let mut reader = std::fs::File::open(&file)?;

        let model = match smf::Model::read(&mut reader) {
            Ok(model) => model,
            Err(err) => {
                eprintln!("Could not read model: {}", file.display());
                return Err(err);
            }
        };

        println!("Model({}) | unknown: {:?}", model.name, model.scale);

        fn print_nodes(nodes: &[smf::Node], parent_name: &str, indent: u32) {
            for node in nodes.iter().filter(|node| node.parent_name == parent_name) {
                for _ in 0..indent {
                    print!("  ");
                }
                let tree_id = if node.tree_id == u32::MAX {
                    String::from("n/a")
                } else {
                    format!("{}", node.tree_id)
                };

                println!(
                    "Node({:}) parent: {}, tree_id: {}, position: {:?}, rotation: {:?}",
                    node.name, node.parent_name, tree_id, node.position, node.rotation,
                );

                for mesh in &node.meshes {
                    for _ in 0..indent {
                        print!("  ");
                    }
                    println!(
                        "..Mesh({}) texture: {}, vertices: {}, faces: {}",
                        mesh.name,
                        mesh.texture_name,
                        mesh.vertices.len(),
                        mesh.faces.len()
                    );

                    /*
                    for vertex in &mesh.vertices {
                        for _ in 0..indent + 1 {
                            print!("  ");
                        }
                        println!(
                            "{}: ({}, {}, {}) ",
                            vertex.index, vertex.position.x, vertex.position.y, vertex.position.z
                        );
                    }
                    for face in &mesh.faces {
                        for _ in 0..indent + 1 {
                            print!("  ");
                        }
                        println!(
                            "{}: {}, {}, {} ",
                            face.index, face.indices[0], face.indices[1], face.indices[2],
                        );
                    }
                    */
                }

                for bounding_box in &node.bounding_boxes {
                    for _ in 0..indent {
                        print!("  ");
                    }
                    println!(
                        "..CollisionBox min: {:?}, max: {:?}, unknown: {}",
                        bounding_box.min, bounding_box.max, bounding_box.u0
                    );
                }

                print_nodes(nodes, &node.name, indent + 1);
            }
        }

        print_nodes(&model.nodes, "<root>", 1);
    }

    Ok(())
}
