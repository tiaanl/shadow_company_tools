use clap::Parser;
use shadow_company_tools::smf;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Opts {
    /// Path to the .smf file you want to operate on.
    path: PathBuf,
    /// Print out mesh vertices, index and faces.
    #[arg(long, short)]
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

        let mut tree = ptree::TreeBuilder::new(file.display().to_string());

        tree.begin_child(format!(
            "Model({}) | unknown: {:?}",
            model.name, model.scale
        ));

        fn print_nodes(
            nodes: &[smf::Node],
            parent_name: &str,
            tree: &mut ptree::TreeBuilder,
            print_mesh_details: bool,
        ) {
            for node in nodes.iter().filter(|node| node.parent_name == parent_name) {
                let tree_id = if node.tree_id == u32::MAX {
                    String::from("n/a")
                } else {
                    format!("{}", node.tree_id)
                };

                tree.begin_child(format!(
                    "Node({:}) tree_id: {}, position: {:?}, rotation: {:?}",
                    node.name, tree_id, node.position, node.rotation,
                ));

                if !node.meshes.is_empty() {
                    tree.begin_child("Meshes".to_string());
                    for mesh in &node.meshes {
                        tree.begin_child(format!(
                            "Mesh({}) texture: {}, vertices: {}, faces: {}",
                            mesh.name,
                            mesh.texture_name,
                            mesh.vertices.len(),
                            mesh.faces.len()
                        ));

                        if print_mesh_details {
                            if !mesh.vertices.is_empty() {
                                tree.begin_child("Vertices".to_string());
                                for vertex in &mesh.vertices {
                                    tree.add_empty_child(format!(
                                        "{:4}: ({:9.2}, {:9.2}, {:9.2}) ({:9.2}, {:9.2}) ({:9.2}, {:9.2}, {:9.2})",
                                        vertex.index,
                                        vertex.position.x,
                                        vertex.position.y,
                                        vertex.position.z,
                                        vertex.tex_coord.x,
                                        vertex.tex_coord.y,
                                        vertex.normal.x,
                                        vertex.normal.y,
                                        vertex.normal.z,
                                    ));
                                }
                                tree.end_child();
                            }
                            if !mesh.faces.is_empty() {
                                tree.begin_child("Faces".to_string());
                                for face in &mesh.faces {
                                    tree.add_empty_child(format!(
                                        "{:4}: {:3}, {:3}, {:3} ",
                                        face.index,
                                        face.indices[0],
                                        face.indices[1],
                                        face.indices[2],
                                    ));
                                }
                                tree.end_child();
                            }
                        }

                        tree.end_child();
                    }
                    tree.end_child();

                    if print_mesh_details && !node.bounding_boxes.is_empty() {
                        tree.begin_child("Bounding Boxes".to_string());
                        for bounding_box in &node.bounding_boxes {
                            tree.add_empty_child(format!(
                                "CollisionBox min: {:?}, max: {:?}, unknown: {}",
                                bounding_box.min, bounding_box.max, bounding_box.u0
                            ));
                        }
                        tree.end_child();
                    }
                }

                print_nodes(nodes, &node.name, tree, print_mesh_details);

                tree.end_child();
            }
        }

        print_nodes(&model.nodes, "<root>", &mut tree, opts.print_mesh_details);

        tree.end_child();

        ptree::print_tree(&tree.build()).unwrap();
    }

    Ok(())
}
