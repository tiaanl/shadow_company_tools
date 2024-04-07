use clap::Parser;
use gltf_json as json;
use image::ImageFormat;
use json::{
    material::PbrMetallicRoughness,
    validation::{Checked::Valid, USize64},
};
use shadow_company_tools::smf;
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
}

fn main() {
    let opts = Opts::parse();

    let files = if opts.path.is_file() {
        vec![opts.path]
    } else {
        walkdir::WalkDir::new(opts.path)
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

    for file in files {
        let texture_path = if let Some(ref texture_path) = opts.texture_path {
            texture_path.clone()
        } else {
            file.parent()
                .expect("Could not determine texture path.")
                .to_owned()
        };

        convert(file, texture_path, opts.embed_images).expect("could not convert file");
    }
}

fn convert(
    path: impl AsRef<Path>,
    texture_path: impl AsRef<Path>,
    embed_images: bool,
) -> std::io::Result<()> {
    let from_path = path.as_ref().to_owned();
    let to_path = from_path.with_extension("gltf");

    let mut file = std::fs::File::open(&from_path)?;
    let smf = smf::Scene::read(&mut file)?;

    let gltf_json = smf_to_gltf_json(smf, &to_path, texture_path, embed_images);

    let writer = std::fs::File::create(&to_path)?;
    json::serialize::to_writer_pretty(writer, &gltf_json)?;

    println!("Wrote to: {}", to_path.display());

    Ok(())
}

struct VV {
    position: [f32; 3],
    _normal: [f32; 3],
    _uv: [f32; 3],
}

fn smf_to_gltf_json(
    scene: smf::Scene,
    to_path: impl AsRef<Path>,
    texture_path: impl AsRef<Path>,
    embed_images: bool,
) -> json::Root {
    let mut root = json::Root::default();

    let mut root_index = None;
    let mut node_indices = HashMap::new();

    let mut material_indices = HashMap::new();

    for smf_node in scene.nodes.iter() {
        let mut node = json::Node {
            translation: Some([
                smf_node.position.x,
                smf_node.position.z,
                smf_node.position.y,
            ]),
            name: Some(smf_node.name.clone()),
            ..Default::default()
        };

        for smf_mesh in smf_node.meshes.iter() {
            macro_rules! normalize {
                ($v:expr) => {{
                    if $v.is_nan() {
                        0.0
                    } else {
                        $v
                    }
                }};
            }
            let smf_vertices = smf_mesh
                .vertices
                .iter()
                .map(|v| VV {
                    position: [v.position.x, v.position.z, v.position.y],
                    _normal: [
                        -normalize!(v.normal.x),
                        -normalize!(v.normal.z),
                        -normalize!(v.normal.y),
                    ],
                    _uv: [v.tex_coord.x, v.tex_coord.y, 0.0],
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
                let image_path = if let Some(image_path) =
                    find_image_path(&texture_path, &smf_mesh.texture_name)
                {
                    if embed_images {
                        println!("Embedding image: {}", image_path.display());
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
                    double_sided: false,
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
            children.push(*index);
        } else {
            parent.children = Some(vec![*index]);
        }
    }

    let root_index = root_index.expect("no root node found");

    root.push(json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some(scene.name.clone()),
        nodes: vec![root_index],
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
    let file = std::fs::File::open(image_path.as_ref())?;
    let reader = std::io::BufReader::new(file);

    let ext = image_path
        .as_ref()
        .extension()
        .expect("Could not get image extension.")
        .to_str()
        .expect("Could not get image extension as a string.");
    let image_format = ImageFormat::from_extension(ext)
        .unwrap_or_else(|| panic!("Image format not supported ({ext})"));

    let image = image::load(reader, image_format).expect("Could not read image file.");

    let mut buf = Vec::new();
    let mut writer = std::io::Cursor::new(&mut buf);
    image
        .write_to(&mut writer, ImageFormat::Png)
        .expect("Could not convert image.");

    Ok(create_data_uri(&buf, "image/png"))
}
