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
        let mut reader = std::fs::File::open(file)?;
        let model = smf::Model::read(&mut reader)?;

        println!("Model({}) | unknown: {:?}", model.name, model.scale);

        fn print_nodes(nodes: &[smf::Node], parent_name: &str, indent: u32) {
            for node in nodes.iter().filter(|node| node.parent_name == parent_name) {
                for _ in 0..indent {
                    print!("  ");
                }
                let bone_index = if node.bone_index == u32::MAX {
                    String::from("n/a")
                } else {
                    format!("{}", node.bone_index)
                };

                println!(
                    "Node({:}) bone: {}, position: {:?}, rotation: {:?}",
                    node.name, bone_index, node.position, node.rotation,
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
                }

                for collision_box in &node.bounding_boxes {
                    for _ in 0..indent {
                        print!("  ");
                    }
                    println!(
                        "..CollisionBox min: {:?}, max: {:?}, unknown: {}",
                        collision_box.min, collision_box.max, collision_box.u0
                    );
                }

                print_nodes(nodes, &node.name, indent + 1);
            }
        }

        print_nodes(&model.nodes, "<root>", 1);
    }

    Ok(())
}
