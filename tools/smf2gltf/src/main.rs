use clap::{Parser, ValueEnum};
use gltf_json::{self as json, scene::UnitQuaternion};
use image::ImageFormat;
use json::{
    material::PbrMetallicRoughness,
    validation::{Checked::Valid, USize64},
};
use shadow_company_tools::{
    smf::{self, CONVERT, CONVERT_NORMAL},
    Mat4, Quat, Vec3,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Parser)]
struct Opts {
    /// Path to a .smf file or a directory containing .smf files.
    path: PathBuf,
    /// The root path to search for images.
    #[arg(short, long)]
    texture_path: Option<PathBuf>,
    /// Whether to embed images into the .gltf file.
    #[arg(short, long)]
    embed_images: bool,
    #[arg(short, long, default_value = "1.0")]
    scale: f32,
    #[arg(short, long)]
    verbose: bool,
    /// How to export meshes: attach to nodes or export a skeleton with skinning.
    #[arg(long, value_enum, default_value_t = NodeMode::Nodes)]
    node_mode: NodeMode,
    /// Apply SMF rotations to skeleton joints (enable with --skeleton-rotations).
    #[arg(long, default_value_t = false)]
    skeleton_rotations: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum NodeMode {
    Nodes,
    Skeleton,
}

fn main() {
    let opts = Opts::parse();

    let files = if opts.path.is_file() {
        vec![opts.path.clone()]
    } else {
        walkdir::WalkDir::new(&opts.path)
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|p| {
                if p.file_type().is_file() {
                    Some(p.path().to_owned())
                } else {
                    None
                }
            })
            .filter(|p| {
                if let Some(ext) = p.extension() {
                    ext.eq_ignore_ascii_case("smf")
                } else {
                    false
                }
            })
            .collect()
    };

    files
        .iter()
        .for_each(|file| convert(file, &opts).expect("Could not export file."));
}

fn convert(path: impl AsRef<Path>, opts: &Opts) -> std::io::Result<()> {
    let from_path = path.as_ref().to_owned();
    let to_path = from_path.with_extension("gltf");

    let mut file = std::fs::File::open(&from_path)?;
    let smf = smf::Model::read(&mut file)?;

    let gltf_json = smf_to_gltf_json(smf, &to_path, opts);

    let writer = std::fs::File::create(&to_path)?;
    json::serialize::to_writer_pretty(writer, &gltf_json)?;

    // println!("Wrote to: {}", to_path.display());

    Ok(())
}

struct VV {
    position: [f32; 3],
    _normal: [f32; 3],
    _uv: [f32; 3],
}

struct NodeTransform {
    local_matrix: Mat4,
}

fn smf_to_gltf_json(scene: smf::Model, to_path: impl AsRef<Path>, opts: &Opts) -> json::Root {
    let mut root = json::Root::default();

    let mut root_index = None;
    let mut node_indices = HashMap::new();

    let mut material_indices = HashMap::new();

    let skeleton_mode = matches!(opts.node_mode, NodeMode::Skeleton);
    let use_joint_rotations = skeleton_mode && opts.skeleton_rotations;

    let convert_position = |v: Vec3| {
        let m = CONVERT * Mat4::from_scale(Vec3::splat(opts.scale));
        m.project_point3(v)
    };

    let convert_rotation = |q: Quat| {
        let rotation_z_to_y = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        let transformed_quaternion = rotation_z_to_y * q;
        Quat::from_xyzw(
            -transformed_quaternion.x,
            transformed_quaternion.y,
            transformed_quaternion.z,
            transformed_quaternion.w,
        )
    };

    let mut name_to_smf_index = HashMap::new();
    for (index, smf_node) in scene.nodes.iter().enumerate() {
        name_to_smf_index.insert(smf_node.name.clone(), index);
    }

    let mut node_translations = Vec::with_capacity(scene.nodes.len());
    let mut joint_rotations = Vec::new();
    let mut parent_indices = Vec::with_capacity(scene.nodes.len());
    for smf_node in scene.nodes.iter() {
        let translation = convert_position(smf_node.position);
        node_translations.push(translation);

        if skeleton_mode {
            let joint_rotation = if use_joint_rotations {
                convert_rotation(smf_node.rotation)
            } else {
                Quat::IDENTITY
            };
            joint_rotations.push(joint_rotation);
        }

        let parent_index = if smf_node.parent_name == "<root>" {
            None
        } else {
            Some(
                *name_to_smf_index
                    .get(&smf_node.parent_name)
                    .expect("parent node not found"),
            )
        };
        parent_indices.push(parent_index);
    }

    let mut joint_translations = Vec::new();
    let mut joint_transforms = Vec::new();
    let mut mesh_transforms = Vec::new();
    if skeleton_mode {
        joint_translations = if use_joint_rotations {
            compute_joint_translations(&node_translations, &joint_rotations, &parent_indices)
        } else {
            node_translations.clone()
        };

        joint_transforms.reserve(scene.nodes.len());
        mesh_transforms.reserve(scene.nodes.len());
        for index in 0..scene.nodes.len() {
            let joint_rotation = joint_rotations[index];
            let joint_translation = joint_translations[index];
            let joint_local_matrix =
                Mat4::from_rotation_translation(joint_rotation, joint_translation);
            joint_transforms.push(NodeTransform {
                local_matrix: joint_local_matrix,
            });

            let mesh_local_matrix = Mat4::from_translation(node_translations[index]);
            mesh_transforms.push(NodeTransform {
                local_matrix: mesh_local_matrix,
            });
        }
    }

    let joint_global_transforms = if skeleton_mode {
        Some(compute_global_transforms(&joint_transforms, &parent_indices))
    } else {
        None
    };

    let mesh_global_transforms = if skeleton_mode {
        Some(compute_global_transforms(&mesh_transforms, &parent_indices))
    } else {
        None
    };

    if skeleton_mode && scene.nodes.len() > u16::MAX as usize {
        panic!("Too many joints for u16 joint indices.");
    }

    let mut joint_nodes = Vec::with_capacity(scene.nodes.len());
    for (node_i, smf_node) in scene.nodes.iter().enumerate() {
        let mut node = json::Node {
            translation: Some(
                if skeleton_mode {
                    joint_translations[node_i]
                } else {
                    node_translations[node_i]
                }
                .to_array(),
            ),
            name: Some(smf_node.name.clone()),
            ..Default::default()
        };

        if skeleton_mode {
            node.rotation = Some(UnitQuaternion(joint_rotations[node_i].to_array()));
        }

        let node_index = root.push(node);
        node_indices.insert(smf_node.name.clone(), node_index);
        joint_nodes.push(node_index);
        if smf_node.parent_name == "<root>" {
            root_index = Some(node_index);
        }
    }

    for smf_node in scene.nodes.iter() {
        if smf_node.parent_name == "<root>" {
            continue;
        }

        let Some(index) = node_indices.get(&smf_node.name) else {
            panic!("node index not found");
        };

        let Some(parent_index) = node_indices.get(&smf_node.parent_name) else {
            panic!("parent index not found!");
        };

        let Some(parent) = root.nodes.get_mut(parent_index.value()) else {
            panic!("parent node not found!");
        };

        if let Some(ref mut children) = parent.children {
            children.push(*index);
        } else {
            parent.children = Some(vec![*index]);
        }
    }

    let root_index = root_index.expect("no root node found");

    let skin_index = if skeleton_mode {
        let joint_global_transforms = joint_global_transforms
            .as_ref()
            .expect("missing joint global transforms");
        let inverse_bind_matrices = joint_global_transforms
            .iter()
            .map(|transform| transform.inverse().to_cols_array())
            .collect::<Vec<_>>();
        let inverse_bind_count = inverse_bind_matrices.len();

        let buffer = create_buffer(inverse_bind_matrices);
        let byte_length = buffer.byte_length;
        let buffer = root.push(buffer);

        let buffer_view = root.push(json::buffer::View {
            buffer,
            byte_length,
            byte_offset: None,
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: None,
        });

        let inverse_bind_matrices = root.push(json::Accessor {
            buffer_view: Some(buffer_view),
            byte_offset: Some(USize64(0)),
            count: USize64::from(inverse_bind_count),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Mat4),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        Some(root.push(json::Skin {
            inverse_bind_matrices: Some(inverse_bind_matrices),
            skeleton: Some(root_index),
            joints: joint_nodes.clone(),
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        }))
    } else {
        None
    };

    let mut mesh_node_indices = Vec::new();

    for (node_i, smf_node) in scene.nodes.iter().enumerate() {
        let node_global_transform = if skeleton_mode {
            Some(
                mesh_global_transforms
                    .as_ref()
                    .expect("missing mesh global transforms")[node_i],
            )
        } else {
            None
        };
        let joint_index = if skeleton_mode {
            Some(u16::try_from(node_i).expect("Joint index exceeds u16."))
        } else {
            None
        };

        for smf_mesh in smf_node.meshes.iter() {
            let smf_vertices = smf_mesh
                .vertices
                .iter()
                .map(|v| {
                    let mut position = convert_position(v.position);
                    let mut normal = CONVERT_NORMAL.project_point3(-v.normal);

                    if let Some(transform) = node_global_transform {
                        position = transform.transform_point3(position);
                        normal = transform.transform_vector3(normal).normalize();
                    }

                    VV {
                        position: position.to_array(),
                        _normal: normal.to_array(),
                        _uv: [v.tex_coord.x, v.tex_coord.y, 0.0],
                    }
                })
                .collect::<Vec<_>>();
            let vertex_count = smf_vertices.len();

            let (min, max) = bounding_coords(&smf_vertices);

            let vertices_buffer_view = {
                let buffer = create_buffer(smf_vertices);
                let byte_length = buffer.byte_length;
                let buffer = root.push(buffer);

                root.push(json::buffer::View {
                    buffer,
                    byte_length,
                    byte_offset: None,
                    byte_stride: Some(json::buffer::Stride(std::mem::size_of::<VV>())),
                    extensions: Default::default(),
                    extras: Default::default(),
                    name: None,
                    target: Some(Valid(json::buffer::Target::ArrayBuffer)),
                })
            };

            let positions = root.push(json::Accessor {
                buffer_view: Some(vertices_buffer_view),
                byte_offset: Some(USize64(0)),
                count: USize64::from(vertex_count),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec3),
                min: Some(json::Value::from(Vec::from(min))),
                max: Some(json::Value::from(Vec::from(max))),
                name: None,
                normalized: false,
                sparse: None,
            });

            let normals = root.push(json::Accessor {
                buffer_view: Some(vertices_buffer_view),
                byte_offset: Some(USize64::from(3 * std::mem::size_of::<f32>())),
                count: USize64::from(vertex_count),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec3),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            let uvs = root.push(json::Accessor {
                buffer_view: Some(vertices_buffer_view),
                byte_offset: Some(USize64::from(6 * std::mem::size_of::<f32>())),
                count: USize64::from(vertex_count),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec2),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            let (joints, weights) = if let Some(joint_index) = joint_index {
                let joints_data = vec![[joint_index, 0, 0, 0]; vertex_count];
                let joints_buffer = create_buffer(joints_data);
                let joints_byte_length = joints_buffer.byte_length;
                let joints_buffer = root.push(joints_buffer);
                let joints_view = root.push(json::buffer::View {
                    buffer: joints_buffer,
                    byte_length: joints_byte_length,
                    byte_offset: None,
                    byte_stride: None,
                    name: None,
                    target: Some(Valid(json::buffer::Target::ArrayBuffer)),
                    extensions: Default::default(),
                    extras: Default::default(),
                });

                let joints_accessor = root.push(json::Accessor {
                    buffer_view: Some(joints_view),
                    byte_offset: Some(USize64(0)),
                    count: USize64::from(vertex_count),
                    component_type: Valid(json::accessor::GenericComponentType(
                        json::accessor::ComponentType::U16,
                    )),
                    extensions: Default::default(),
                    extras: Default::default(),
                    type_: Valid(json::accessor::Type::Vec4),
                    min: None,
                    max: None,
                    name: None,
                    normalized: false,
                    sparse: None,
                });

                let weights_data = vec![[1.0_f32, 0.0, 0.0, 0.0]; vertex_count];
                let weights_buffer = create_buffer(weights_data);
                let weights_byte_length = weights_buffer.byte_length;
                let weights_buffer = root.push(weights_buffer);
                let weights_view = root.push(json::buffer::View {
                    buffer: weights_buffer,
                    byte_length: weights_byte_length,
                    byte_offset: None,
                    byte_stride: None,
                    name: None,
                    target: Some(Valid(json::buffer::Target::ArrayBuffer)),
                    extensions: Default::default(),
                    extras: Default::default(),
                });

                let weights_accessor = root.push(json::Accessor {
                    buffer_view: Some(weights_view),
                    byte_offset: Some(USize64(0)),
                    count: USize64::from(vertex_count),
                    component_type: Valid(json::accessor::GenericComponentType(
                        json::accessor::ComponentType::F32,
                    )),
                    extensions: Default::default(),
                    extras: Default::default(),
                    type_: Valid(json::accessor::Type::Vec4),
                    min: None,
                    max: None,
                    name: None,
                    normalized: false,
                    sparse: None,
                });

                (Some(joints_accessor), Some(weights_accessor))
            } else {
                (None, None)
            };

            let indices = {
                let indices = smf_mesh
                    .faces
                    .iter()
                    .flat_map(|f| [f.indices[2], f.indices[1], f.indices[0]])
                    .collect::<Vec<_>>();
                let indices_count = indices.len();

                let buffer = create_buffer(indices);
                let byte_length = buffer.byte_length;
                let buffer = root.push(buffer);

                let buffer_view = root.push(json::buffer::View {
                    buffer,
                    byte_length,
                    byte_offset: None,
                    byte_stride: None, // No byte stride for indices.
                    name: None,
                    target: Some(Valid(json::buffer::Target::ElementArrayBuffer)),
                    extensions: Default::default(),
                    extras: Default::default(),
                });

                root.push(json::Accessor {
                    buffer_view: Some(buffer_view),
                    byte_offset: Some(USize64::from(0_u64)),
                    count: USize64::from(indices_count),
                    component_type: Valid(json::accessor::GenericComponentType(
                        json::accessor::ComponentType::U32,
                    )),
                    extensions: Default::default(),
                    extras: Default::default(),
                    type_: Valid(json::accessor::Type::Scalar),
                    min: None,
                    max: None,
                    name: None,
                    normalized: false,
                    sparse: None,
                })
            };

            let material_i = if let Some(mat) = material_indices.get(&smf_mesh.texture_name) {
                *mat
            } else {
                let texture_path = if let Some(ref texture_path) = opts.texture_path {
                    texture_path.clone()
                } else {
                    to_path
                        .as_ref()
                        .parent()
                        .expect("Could not get directory parent.")
                        .to_path_buf()
                };
                let image_path = if let Some(image_path) =
                    find_image_path(&texture_path, &smf_mesh.texture_name)
                {
                    if opts.embed_images {
                        // println!("Embedding image: {}", image_path.display());
                        image_to_buffer(image_path).expect("Could not embed image.")
                    } else {
                        pathdiff::diff_paths(&image_path, to_path.as_ref())
                            .expect("Could not determine relative texture path.")
                            .to_str()
                            .unwrap()
                            .to_owned()
                    }
                } else {
                    eprintln!("Warning: Could not find image: {}", smf_mesh.texture_name);
                    PathBuf::from(&smf_mesh.texture_name)
                        .to_str()
                        .unwrap()
                        .to_owned()
                };

                let image_i = root.push(json::Image {
                    buffer_view: None,
                    mime_type: None,
                    name: Some(smf_mesh.name.clone()),
                    uri: Some(image_path),
                    extensions: None,
                    extras: None,
                });

                let texture_i = root.push(json::Texture {
                    name: None,
                    sampler: None,
                    source: image_i,
                    extensions: None,
                    extras: None,
                });

                let material_i = root.push(json::Material {
                    alpha_cutoff: None,
                    alpha_mode: Valid(json::material::AlphaMode::Opaque),
                    double_sided: true,
                    name: None,
                    pbr_metallic_roughness: PbrMetallicRoughness {
                        base_color_factor: json::material::PbrBaseColorFactor([1.0, 1.0, 1.0, 1.0]),
                        base_color_texture: Some(json::texture::Info {
                            index: texture_i,
                            tex_coord: 0,
                            extensions: None,
                            extras: None,
                        }),
                        metallic_factor: json::material::StrengthFactor(0.0),
                        roughness_factor: json::material::StrengthFactor(1.0),
                        metallic_roughness_texture: None,
                        extensions: None,
                        extras: None,
                    },
                    normal_texture: None,
                    occlusion_texture: None,
                    emissive_texture: None,
                    emissive_factor: json::material::EmissiveFactor([0.0, 0.0, 0.0]),
                    extensions: None,
                    extras: None,
                });

                material_indices.insert(smf_mesh.texture_name.clone(), material_i);

                material_i
            };

            let primitive = json::mesh::Primitive {
                attributes: {
                    let mut map = std::collections::BTreeMap::new();
                    map.insert(Valid(json::mesh::Semantic::Positions), positions);
                    map.insert(Valid(json::mesh::Semantic::Normals), normals);
                    map.insert(Valid(json::mesh::Semantic::TexCoords(0)), uvs);
                    if let Some(joints) = joints {
                        map.insert(Valid(json::mesh::Semantic::Joints(0)), joints);
                    }
                    if let Some(weights) = weights {
                        map.insert(Valid(json::mesh::Semantic::Weights(0)), weights);
                    }
                    map
                },
                extensions: Default::default(),
                extras: Default::default(),
                indices: Some(indices),
                material: Some(material_i),
                mode: Valid(json::mesh::Mode::Triangles),
                targets: None,
            };

            let mesh_index = root.push(json::Mesh {
                extensions: Default::default(),
                extras: Default::default(),
                name: Some(smf_mesh.name.clone()),
                primitives: vec![primitive],
                weights: None,
            });

            let node_index = root.push(json::Node {
                mesh: Some(mesh_index),
                name: Some(smf_mesh.name.clone()),
                skin: skin_index,
                ..Default::default()
            });

            if skeleton_mode {
                mesh_node_indices.push(node_index);
            } else {
                let Some(parent_index) = node_indices.get(&smf_node.name) else {
                    panic!("node index not found");
                };

                let Some(parent) = root.nodes.get_mut(parent_index.value()) else {
                    panic!("parent node not found!");
                };

                if let Some(ref mut children) = parent.children {
                    children.push(node_index);
                } else {
                    parent.children = Some(vec![node_index]);
                }
            }
        }
    }

    let mut scene_nodes = vec![root_index];
    if skeleton_mode {
        scene_nodes.extend(mesh_node_indices);
    }

    root.push(json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some(scene.name.clone()),
        nodes: scene_nodes,
    });

    root
}

fn find_image_path(root: impl AsRef<Path>, name: &str) -> Option<PathBuf> {
    walkdir::WalkDir::new(root.as_ref())
        .into_iter()
        .filter_map(Result::ok)
        .find(|e| e.file_name().eq_ignore_ascii_case(name))
        .map(|e| e.into_path())
}

/// Calculate bounding coordinates of a list of vertices, used for the clipping distance of the model
fn bounding_coords(points: &[VV]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN];

    for point in points {
        let p = point.position;
        for i in 0..3 {
            min[i] = f32::min(min[i], p[i]);
            max[i] = f32::max(max[i], p[i]);
        }
    }
    (min, max)
}

fn compute_global_transforms(
    transforms: &[NodeTransform],
    parent_indices: &[Option<usize>],
) -> Vec<Mat4> {
    let mut globals = vec![Mat4::IDENTITY; transforms.len()];
    let mut visited = vec![false; transforms.len()];

    fn visit(
        index: usize,
        transforms: &[NodeTransform],
        parent_indices: &[Option<usize>],
        globals: &mut [Mat4],
        visited: &mut [bool],
    ) {
        if visited[index] {
            return;
        }

        let global = if let Some(parent_index) = parent_indices[index] {
            visit(parent_index, transforms, parent_indices, globals, visited);
            globals[parent_index] * transforms[index].local_matrix
        } else {
            transforms[index].local_matrix
        };

        globals[index] = global;
        visited[index] = true;
    }

    for index in 0..transforms.len() {
        visit(index, transforms, parent_indices, &mut globals, &mut visited);
    }

    globals
}

fn compute_joint_translations(
    translations: &[Vec3],
    rotations: &[Quat],
    parent_indices: &[Option<usize>],
) -> Vec<Vec3> {
    let mut adjusted = vec![Vec3::ZERO; translations.len()];
    let mut global_rotations = vec![Quat::IDENTITY; translations.len()];
    let mut visited = vec![false; translations.len()];

    fn visit(
        index: usize,
        translations: &[Vec3],
        rotations: &[Quat],
        parent_indices: &[Option<usize>],
        adjusted: &mut [Vec3],
        global_rotations: &mut [Quat],
        visited: &mut [bool],
    ) {
        if visited[index] {
            return;
        }

        if let Some(parent_index) = parent_indices[index] {
            visit(
                parent_index,
                translations,
                rotations,
                parent_indices,
                adjusted,
                global_rotations,
                visited,
            );
            let parent_global_rotation = global_rotations[parent_index];
            adjusted[index] = parent_global_rotation.inverse() * translations[index];
            global_rotations[index] = parent_global_rotation * rotations[index];
        } else {
            adjusted[index] = translations[index];
            global_rotations[index] = rotations[index];
        }

        visited[index] = true;
    }

    for index in 0..translations.len() {
        visit(
            index,
            translations,
            rotations,
            parent_indices,
            &mut adjusted,
            &mut global_rotations,
            &mut visited,
        );
    }

    adjusted
}

fn to_padded_byte_vector<T>(vec: Vec<T>) -> Vec<u8> {
    let byte_length = vec.len() * std::mem::size_of::<T>();
    let byte_capacity = vec.capacity() * std::mem::size_of::<T>();
    let alloc = vec.into_boxed_slice();
    let ptr = Box::<[T]>::into_raw(alloc) as *mut u8;
    let mut new_vec = unsafe { Vec::from_raw_parts(ptr, byte_length, byte_capacity) };
    while new_vec.len() % 4 != 0 {
        new_vec.push(0); // pad to multiple of four bytes
    }
    new_vec
}

fn create_buffer<T>(buffer: Vec<T>) -> json::Buffer {
    let count = buffer.len();
    let byte_length = count * std::mem::size_of::<T>();
    let byte_vector = to_padded_byte_vector(buffer);

    let data_uri = create_data_uri(&byte_vector, "application/octet-stream");

    json::Buffer {
        byte_length: USize64::from(byte_length),
        name: None,
        uri: Some(data_uri),
        extensions: None,
        extras: None,
    }
}

fn create_data_uri(data: &[u8], mime_type: &str) -> String {
    use base64::Engine;

    let base64 = base64::engine::GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        base64::engine::general_purpose::PAD,
    );

    let encoded_buffer = base64.encode(data);

    format!("data:{mime_type};base64,{encoded_buffer}")
}

fn image_to_buffer(image_path: impl AsRef<Path>) -> std::io::Result<String> {
    println!("image_path: {}", image_path.as_ref().display());
    let file = std::fs::File::open(image_path.as_ref())?;
    let mut reader = std::io::BufReader::new(file);

    let ext = image_path.as_ref().extension().unwrap();
    if !ext.eq_ignore_ascii_case("bmp") {
        panic!("Invalid image format!");
    }

    // TODO: Check if the image is color keyed.
    let bmp_image = shadow_company_tools::images::load_bmp_file(&mut reader, false)
        .expect("Could not open .bmp file");

    let raw_path = image_path.as_ref().with_extension("raw");
    let png = if raw_path.exists() {
        println!("raw_path: {}", raw_path.display());
        let raw_image = shadow_company_tools::images::load_raw_file(
            &mut std::fs::File::open(&raw_path).expect("Could not open .raw file."),
            bmp_image.width(),
            bmp_image.height(),
        )
        .expect("Could not read .raw file.");
        shadow_company_tools::images::combine_bmp_and_raw(&bmp_image, &raw_image)
    } else {
        use image::buffer::ConvertBuffer;
        let png: image::RgbaImage = bmp_image.convert();
        png
    };

    let mut buf = Vec::new();
    let mut writer = std::io::Cursor::new(&mut buf);
    png.write_to(&mut writer, ImageFormat::Png)
        .expect("Could not generate image buffer.");

    Ok(create_data_uri(&buf, "image/png"))
}
