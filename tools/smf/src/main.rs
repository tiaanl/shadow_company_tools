use base64::Engine;
use clap::Parser;
use json::material::PbrMetallicRoughness;
use shadow_company_tools::smf;
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

    /// Apply a global scale to the model.
    #[arg(short, long, default_value = "1.0")]
    scale: f32,
}

struct VV {
    position: [f32; 3],
    _normal: [f32; 3],
    _uv: [f32; 3],
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

    let base64 = base64::engine::GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        base64::engine::general_purpose::PAD,
    );

    let encoded_buffer = base64.encode(&byte_vector);

    json::Buffer {
        byte_length: USize64::from(byte_length),
        name: None,
        uri: Some(format!(
            "data:application/octet-stream;base64,{}",
            encoded_buffer
        )),
        extensions: None,
        extras: None,
    }
}

fn main() {
    let opts = Opts::parse();
    let scale = opts.scale;

    let mut c = Cursor::new(std::fs::read(&opts.path).unwrap());

    shadow_company_tools::common::skip_sinister_header(&mut c).unwrap();

    let scene = smf::Scene::read(&mut c);

    println!("Model: {}, scale: {:?}", scene.name, scene.scale);
    scene.nodes.iter().for_each(|model| {
        println!(
            "  Node: {} ({}), position: {:?}, rotation: {:?}",
            model.name, model.parent_name, model.position, model.rotation
        );
        model.meshes.iter().for_each(|m| {
            println!("    Mesh: {}, vertices: {}", m.name, m.vertices.len());
        });
    });

    let out_path = opts.path.with_extension("gltf");

    let mut root = json::Root::default();

    let mut root_index = None;
    let mut node_indices = HashMap::new();

    for smf_node in scene.nodes.iter() {
        let mut node = json::Node {
            translation: Some(
                [
                    smf_node.position.0,
                    smf_node.position.1,
                    smf_node.position.2,
                ]
                .map(|v| v * scale),
            ),
            name: Some(smf_node.name.clone()),
            ..Default::default()
        };

        for smf_mesh in smf_node.meshes.iter() {
            let smf_vertices = smf_mesh
                .vertices
                .iter()
                .map(|v| VV {
                    position: [v.position.0, v.position.1, v.position.2].map(|v| v * scale),
                    _normal: [-v.normal.0, -v.normal.1, -v.normal.2],
                    _uv: [v.tex_coord.0, v.tex_coord.1, 0.0],
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

            let indices = {
                let indices = smf_mesh
                    .faces
                    .iter()
                    .flat_map(|f| [f.indices[0], f.indices[1], f.indices[2]])
                    .collect::<Vec<_>>();
                let indices_count = indices.len();

                let buffer = create_buffer(indices);
                let byte_length = buffer.byte_length;
                let buffer = root.push(buffer);

                let buffer_view = root.push(json::buffer::View {
                    buffer,
                    byte_length,
                    byte_offset: None,
                    byte_stride: Some(json::buffer::Stride(std::mem::size_of::<u32>())),
                    name: None,
                    target: Some(Valid(json::buffer::Target::ArrayBuffer)),
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

            let image = root.push(json::Image {
                buffer_view: None,
                mime_type: None,
                name: Some(smf_mesh.name.clone()),
                uri: Some(format!("../../textures/shared/{}", smf_mesh.texture_name)),
                extensions: None,
                extras: None,
            });

            let texture = root.push(json::Texture {
                name: None,
                sampler: None,
                source: image,
                extensions: None,
                extras: None,
            });

            let material = root.push(json::Material {
                alpha_cutoff: None,
                alpha_mode: Valid(json::material::AlphaMode::Opaque),
                double_sided: false,
                name: None,
                pbr_metallic_roughness: PbrMetallicRoughness {
                    base_color_factor: json::material::PbrBaseColorFactor([1.0, 1.0, 1.0, 1.0]),
                    base_color_texture: Some(json::texture::Info {
                        index: texture,
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

            let primitive = json::mesh::Primitive {
                attributes: {
                    let mut map = std::collections::BTreeMap::new();
                    map.insert(Valid(json::mesh::Semantic::Positions), positions);
                    map.insert(Valid(json::mesh::Semantic::Normals), normals);
                    map.insert(Valid(json::mesh::Semantic::TexCoords(0)), uvs);
                    map
                },
                extensions: Default::default(),
                extras: Default::default(),
                indices: Some(indices),
                material: Some(material),
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
                ..Default::default()
            });

            if let Some(ref mut children) = node.children {
                children.push(node_index);
            } else {
                node.children = Some(vec![node_index]);
            }
        }

        let node_index = root.push(node);
        node_indices.insert(smf_node.name.clone(), node_index);
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
            children.push(index.clone());
        } else {
            parent.children = Some(vec![index.clone()]);
        }
    }

    let root_index = root_index.expect("no root node found");

    root.push(json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some(scene.name.clone()),
        nodes: vec![root_index],
    });

    let writer = std::fs::File::create(out_path).expect("I/O error");
    json::serialize::to_writer_pretty(writer, &root).expect("Serialization error");
}
