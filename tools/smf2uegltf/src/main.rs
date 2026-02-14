use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use clap::Parser;
use gltf_json as json;
use image::ImageFormat;
use json::{
    accessor,
    animation::{Channel, Interpolation, Property, Sampler, Target},
    buffer, material,
    scene::UnitQuaternion,
    validation::Checked::Valid,
    Accessor, Animation, Buffer, Index, Mesh, Node, Root, Scene, Skin,
};
use shadow_company_tools::{
    bmf,
    smf::{self, CONVERT, CONVERT_NORMAL},
    Mat4, Quat, Vec3, Vec4,
};

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum ExportMode {
    /// Mesh + skin + animation.
    Full,
    /// Mesh + skin only.
    Mesh,
    /// Skeleton + animation only (no mesh).
    Anim,
}

#[derive(Parser)]
struct Opts {
    /// Model file containing the skeleton and mesh.
    smf_path: PathBuf,
    /// Motion file containing animation data (optional when --mode mesh).
    bmf_path: Option<PathBuf>,
    /// Export mode: full, mesh, anim.
    #[arg(long, value_enum, default_value_t = ExportMode::Full)]
    mode: ExportMode,
}

/// BMF data appears to be right-handed z-up; this basis converts to right-handed y-up without an X mirror.
const CONVERT_BMF: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, -1.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.0, 1.0),
);

#[derive(Clone, Copy, Debug)]
struct JointInfo {
    joint_index: u16,
    global_bind: Mat4,
}

fn main() {
    let opts = Opts::parse();

    let smf = smf::Model::read(
        &mut std::fs::File::open(&opts.smf_path).expect("Could not open .smf file."),
    )
    .expect("Could not parse .smf file.");

    let bmf_path = if opts.mode == ExportMode::Mesh {
        opts.bmf_path.clone()
    } else {
        Some(
            opts.bmf_path
                .clone()
                .unwrap_or_else(|| missing_bmf_exit(opts.mode)),
        )
    };

    let bmf = if opts.mode == ExportMode::Mesh {
        None
    } else {
        let bmf_path = bmf_path
            .as_ref()
            .expect("bmf path required for animation export.");
        Some(
            bmf::Motion::read(
                &mut std::fs::File::open(bmf_path).expect("Could not open .bmf file."),
            )
            .expect("Could not parse .bmf file."),
        )
    };

    let out_path = output_path_for_mode(&opts.smf_path, bmf_path.as_deref(), opts.mode);

    println!("out_path: {out_path:?}");

    let gltf = build_gltf(&smf, bmf.as_ref(), &out_path, &opts.smf_path, opts.mode);

    let writer = std::fs::File::create(&out_path).expect("Could not create output file.");
    json::serialize::to_writer_pretty(writer, &gltf).expect("Could not write gltf.");
}

fn build_gltf(
    scene: &smf::Model,
    motion: Option<&bmf::Motion>,
    out_path: &Path,
    smf_path: &Path,
    mode: ExportMode,
) -> Root {
    let mut root = Root::default();

    let mut bone_lookup = HashMap::new();
    let mut joint_info = HashMap::<u32, JointInfo>::new();
    let mut joints = Vec::new();
    let mut joint_globals = Vec::new();
    let scale = 1.0;
    let fps = 30.0;
    let flip_forward = true;
    let invert_anim_rot = true;
    let root_motion = true;
    let root_motion_bone = None;
    let texture_roots = texture_search_roots(smf_path, out_path);

    assert_eq!(scene.nodes[0].parent_name, "<root>");
    let skeleton_root = add_node(
        &mut root,
        &scene.nodes,
        0,
        Mat4::IDENTITY,
        scale,
        &mut bone_lookup,
        &mut joint_info,
        &mut joints,
        &mut joint_globals,
    );

    let mesh_node = match mode {
        ExportMode::Anim => {
            let (mesh_index, skin_index) =
                build_dummy_mesh_and_skin(&mut root, scene, &joints, &joint_globals, skeleton_root);

            Some(root.push(Node {
                camera: None,
                children: None,
                extensions: None,
                extras: None,
                matrix: None,
                mesh: Some(mesh_index),
                name: Some(format!("{}_dummy", scene.name)),
                rotation: None,
                scale: None,
                skin: Some(skin_index),
                translation: None,
                weights: None,
            }))
        }
        ExportMode::Mesh | ExportMode::Full => {
            let (mesh_index, skin_index) = build_mesh_and_skin(
                &mut root,
                scene,
                &texture_roots,
                scale,
                &joint_info,
                &joints,
                &joint_globals,
                skeleton_root,
            );

            Some(root.push(Node {
                camera: None,
                children: None,
                extensions: None,
                extras: None,
                matrix: None,
                mesh: Some(mesh_index),
                name: Some(format!("{}_mesh", scene.name)),
                rotation: None,
                scale: None,
                skin: Some(skin_index),
                translation: None,
                weights: None,
            }))
        }
    };

    if flip_forward {
        apply_scene_rotation(&mut root, skeleton_root);
    }

    if mode != ExportMode::Mesh {
        let motion = motion.expect("bmf motion required for animation export.");
        let root_motion_bone = if root_motion {
            select_root_motion_bone(scene, motion, root_motion_bone)
        } else {
            None
        };

        add_animation(
            &mut root,
            motion,
            &bone_lookup,
            fps,
            scale,
            invert_anim_rot,
            root_motion_bone,
            skeleton_root,
        );
    }

    let mut scene_nodes = vec![skeleton_root];
    if let Some(mesh_node) = mesh_node {
        scene_nodes.push(mesh_node);
    }

    root.push(Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some(scene.name.clone()),
        nodes: scene_nodes,
    });

    root
}

#[allow(clippy::too_many_arguments)]
fn add_node(
    root: &mut Root,
    nodes: &[smf::Node],
    node_index: usize,
    parent_global: Mat4,
    scale: f32,
    bone_lookup: &mut HashMap<u32, Index<Node>>,
    joint_info: &mut HashMap<u32, JointInfo>,
    joints: &mut Vec<Index<Node>>,
    joint_globals: &mut Vec<Mat4>,
) -> Index<Node> {
    let node = &nodes[node_index];

    let translation = convert_position(node.position, scale);
    let rotation = convert_rotation(node.rotation);
    let local = Mat4::from_translation(translation) * Mat4::from_quat(rotation);
    let global = parent_global * local;

    let children = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.parent_name == node.name)
        .map(|(i, _)| {
            add_node(
                root,
                nodes,
                i,
                global,
                scale,
                bone_lookup,
                joint_info,
                joints,
                joint_globals,
            )
        })
        .collect::<Vec<_>>();

    let index = root.push(Node {
        camera: None,
        children: if children.is_empty() {
            None
        } else {
            Some(children)
        },
        extensions: None,
        extras: None,
        matrix: None,
        mesh: None,
        name: Some(node.name.clone()),
        rotation: Some(UnitQuaternion(rotation.to_array())),
        scale: None,
        translation: Some(translation.to_array()),
        skin: None,
        weights: None,
    });

    bone_lookup.insert(node.tree_id, index);

    let joint_index = joints.len() as u16;
    joints.push(index);
    joint_globals.push(global);
    joint_info.insert(
        node.tree_id,
        JointInfo {
            joint_index,
            global_bind: global,
        },
    );

    index
}

fn apply_scene_rotation(root: &mut Root, node_index: Index<Node>) {
    let Some(node) = root.nodes.get_mut(node_index.value()) else {
        return;
    };

    let rotation = Quat::from_rotation_y(std::f32::consts::PI);
    let current = node
        .rotation
        .map(|r| Quat::from_array(r.0))
        .unwrap_or(Quat::IDENTITY);
    let combined = rotation * current;

    node.rotation = Some(UnitQuaternion(combined.to_array()));

    if let Some(translation) = node.translation {
        let rotated = rotation * Vec3::from_array(translation);
        node.translation = Some(rotated.to_array());
    }
}

#[allow(clippy::too_many_arguments)]
fn build_mesh_and_skin(
    root: &mut Root,
    scene: &smf::Model,
    texture_roots: &[PathBuf],
    scale: f32,
    joint_info: &HashMap<u32, JointInfo>,
    joints: &[Index<Node>],
    joint_globals: &[Mat4],
    skeleton_root: Index<Node>,
) -> (Index<Mesh>, Index<Skin>) {
    let mut material_indices = HashMap::new();
    let mut primitives = Vec::new();

    for smf_node in scene.nodes.iter() {
        let joint = joint_info
            .get(&smf_node.tree_id)
            .expect("Missing joint for node.");
        let global_bind = joint.global_bind;
        let joint_index = joint.joint_index;

        for smf_mesh in smf_node.meshes.iter() {
            let mut positions = Vec::with_capacity(smf_mesh.vertices.len());
            let mut normals = Vec::with_capacity(smf_mesh.vertices.len());
            let mut uvs = Vec::with_capacity(smf_mesh.vertices.len());
            let mut joints_0 = Vec::with_capacity(smf_mesh.vertices.len());
            let mut weights_0 = Vec::with_capacity(smf_mesh.vertices.len());

            for v in smf_mesh.vertices.iter() {
                let local_position = convert_position(v.position, scale);
                let world_position = global_bind.transform_point3(local_position);
                positions.push(world_position.to_array());

                let local_normal = CONVERT_NORMAL.project_point3(-v.normal);
                let world_normal = global_bind
                    .transform_vector3(local_normal)
                    .normalize_or_zero();
                normals.push(world_normal.to_array());

                uvs.push([v.tex_coord.x, v.tex_coord.y]);
                joints_0.push([joint_index, 0, 0, 0]);
                weights_0.push([1.0, 0.0, 0.0, 0.0]);
            }

            let indices = smf_mesh
                .faces
                .iter()
                .flat_map(|f| [f.indices[2], f.indices[1], f.indices[0]])
                .collect::<Vec<_>>();

            let (min, max) = bounding_coords(&positions);

            let position_accessor = create_vec3_accessor(root, &positions, Some(min), Some(max));
            let normal_accessor = create_vec3_accessor(root, &normals, None, None);
            let uv_accessor = create_vec2_accessor(root, &uvs);
            let joints_accessor = create_u16x4_accessor(root, &joints_0);
            let weights_accessor = create_vec4_accessor(root, &weights_0);
            let index_accessor = create_indices_accessor(root, &indices);

            let material_index =
                resolve_material(root, smf_mesh, texture_roots, &mut material_indices);

            let mut attributes = std::collections::BTreeMap::new();
            attributes.insert(Valid(json::mesh::Semantic::Positions), position_accessor);
            attributes.insert(Valid(json::mesh::Semantic::Normals), normal_accessor);
            attributes.insert(Valid(json::mesh::Semantic::TexCoords(0)), uv_accessor);
            attributes.insert(Valid(json::mesh::Semantic::Joints(0)), joints_accessor);
            attributes.insert(Valid(json::mesh::Semantic::Weights(0)), weights_accessor);

            primitives.push(json::mesh::Primitive {
                attributes,
                extensions: None,
                extras: None,
                indices: Some(index_accessor),
                material: Some(material_index),
                mode: Valid(json::mesh::Mode::Triangles),
                targets: None,
            });
        }
    }

    let mesh_index = root.push(Mesh {
        extensions: None,
        extras: None,
        name: Some(scene.name.clone()),
        primitives,
        weights: None,
    });

    let inverse_bind_matrices = joint_globals
        .iter()
        .map(|m| m.inverse().to_cols_array())
        .collect::<Vec<_>>();

    let ibm_accessor = create_mat4_accessor(root, &inverse_bind_matrices);

    let skin_index = root.push(Skin {
        extensions: None,
        extras: None,
        inverse_bind_matrices: Some(ibm_accessor),
        joints: joints.to_vec(),
        name: Some(format!("{}_skin", scene.name)),
        skeleton: Some(skeleton_root),
    });

    (mesh_index, skin_index)
}

fn build_dummy_mesh_and_skin(
    root: &mut Root,
    scene: &smf::Model,
    joints: &[Index<Node>],
    joint_globals: &[Mat4],
    skeleton_root: Index<Node>,
) -> (Index<Mesh>, Index<Skin>) {
    let positions = vec![[0.0, 0.0, 0.0], [0.01, 0.0, 0.0], [0.0, 0.01, 0.0]];
    let normals = vec![[0.0, 0.0, 1.0]; 3];
    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
    let joints_0 = vec![[0u16, 0, 0, 0]; 3];
    let weights_0 = vec![[1.0, 0.0, 0.0, 0.0]; 3];
    let indices = vec![0u32, 1, 2];

    let position_accessor = create_vec3_accessor(root, &positions, None, None);
    let normal_accessor = create_vec3_accessor(root, &normals, None, None);
    let uv_accessor = create_vec2_accessor(root, &uvs);
    let joints_accessor = create_u16x4_accessor(root, &joints_0);
    let weights_accessor = create_vec4_accessor(root, &weights_0);
    let index_accessor = create_indices_accessor(root, &indices);

    let material_index = root.push(json::Material {
        alpha_cutoff: None,
        alpha_mode: Valid(material::AlphaMode::Opaque),
        double_sided: true,
        name: Some(format!("{}_dummy_mat", scene.name)),
        pbr_metallic_roughness: material::PbrMetallicRoughness {
            base_color_factor: material::PbrBaseColorFactor([1.0, 1.0, 1.0, 1.0]),
            base_color_texture: None,
            metallic_factor: material::StrengthFactor(0.0),
            roughness_factor: material::StrengthFactor(1.0),
            metallic_roughness_texture: None,
            extensions: None,
            extras: None,
        },
        normal_texture: None,
        occlusion_texture: None,
        emissive_texture: None,
        emissive_factor: material::EmissiveFactor([0.0, 0.0, 0.0]),
        extensions: None,
        extras: None,
    });

    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert(Valid(json::mesh::Semantic::Positions), position_accessor);
    attributes.insert(Valid(json::mesh::Semantic::Normals), normal_accessor);
    attributes.insert(Valid(json::mesh::Semantic::TexCoords(0)), uv_accessor);
    attributes.insert(Valid(json::mesh::Semantic::Joints(0)), joints_accessor);
    attributes.insert(Valid(json::mesh::Semantic::Weights(0)), weights_accessor);

    let mesh_index = root.push(Mesh {
        extensions: None,
        extras: None,
        name: Some(format!("{}_dummy", scene.name)),
        primitives: vec![json::mesh::Primitive {
            attributes,
            extensions: None,
            extras: None,
            indices: Some(index_accessor),
            material: Some(material_index),
            mode: Valid(json::mesh::Mode::Triangles),
            targets: None,
        }],
        weights: None,
    });

    let inverse_bind_matrices = joint_globals
        .iter()
        .map(|m| m.inverse().to_cols_array())
        .collect::<Vec<_>>();

    let ibm_accessor = create_mat4_accessor(root, &inverse_bind_matrices);

    let skin_index = root.push(Skin {
        extensions: None,
        extras: None,
        inverse_bind_matrices: Some(ibm_accessor),
        joints: joints.to_vec(),
        name: Some(format!("{}_skin", scene.name)),
        skeleton: Some(skeleton_root),
    });

    (mesh_index, skin_index)
}

#[allow(clippy::too_many_arguments)]
fn add_animation(
    root: &mut Root,
    motion: &bmf::Motion,
    bone_lookup: &HashMap<u32, Index<Node>>,
    fps: f32,
    scale: f32,
    invert_anim_rot: bool,
    root_motion_bone: Option<u32>,
    root_node: Index<Node>,
) {
    let mut animation = Animation {
        extensions: None,
        extras: None,
        channels: vec![],
        name: Some(motion.name.clone()),
        samplers: vec![],
    };

    let mut translations = HashMap::<u32, (Vec<f32>, Vec<[f32; 3]>)>::new();
    let mut rotations = HashMap::<u32, (Vec<f32>, Vec<[f32; 4]>)>::new();

    let root_node_bone = bone_lookup
        .iter()
        .find(|(_, idx)| **idx == root_node)
        .map(|(id, _)| *id);

    let mut root_motion_bone = root_motion_bone;
    if let (Some(root_motion_id), Some(root_node_id)) = (root_motion_bone, root_node_bone) {
        if root_motion_id == root_node_id {
            eprintln!(
                "Warning: root motion bone {} is the skeleton root; skipping root motion extraction.",
                root_motion_id
            );
            root_motion_bone = None;
        }
    }

    let fps = if fps > 0.0 { fps } else { 30.0 };
    let frame_to_seconds = |frame: u32| frame as f32 / fps;

    for key_frame in motion.key_frames.iter() {
        for bone in key_frame.bones.iter() {
            let bone_id = resolve_bone_id(motion, bone.bone_id);
            let time = frame_to_seconds(bone.time);

            if let Some(translation) = bone.position {
                let translation = convert_position_bmf(translation, scale).to_array();
                let translations = translations.entry(bone_id).or_default();
                translations.0.push(time);
                translations.1.push(translation);
            }

            if let Some(rotation) = bone.rotation {
                let mut rotation = convert_rotation_bmf(rotation);
                if invert_anim_rot {
                    rotation = rotation.conjugate();
                }
                let rotations = rotations.entry(bone_id).or_default();
                rotations.0.push(time);
                rotations.1.push(rotation.to_array());
            }
        }
    }

    if translations.is_empty() && rotations.is_empty() {
        return;
    }

    let mut root_motion = None;
    if let Some(source_bone) = root_motion_bone {
        if let Some((times, values)) = translations.get_mut(&source_bone) {
            if let Some(base) = values.first().copied() {
                let deltas = values
                    .iter()
                    .map(|v| [v[0] - base[0], v[1] - base[1], v[2] - base[2]])
                    .collect::<Vec<_>>();
                let root_times = times.clone();
                for value in values.iter_mut() {
                    *value = base;
                }
                root_motion = Some((root_times, deltas));
            }
        } else {
            eprintln!(
                "Warning: root motion bone {} not found in translation tracks.",
                source_bone
            );
        }
    }

    let root_motion_applied = root_motion.is_some();

    if let Some((times, values)) = root_motion {
        let times_accessor = create_scalar_accessor(root, &times);
        let values_accessor = create_vec3_accessor(root, &values, None, None);

        let sampler = Index::push(
            &mut animation.samplers,
            Sampler {
                extensions: None,
                extras: None,
                input: times_accessor,
                interpolation: Valid(Interpolation::Linear),
                output: values_accessor,
            },
        );

        animation.channels.push(Channel {
            sampler,
            target: Target {
                extensions: None,
                extras: None,
                node: root_node,
                path: Valid(Property::Translation),
            },
            extensions: None,
            extras: None,
        });
    }

    for (bone, (times, values)) in translations.iter() {
        let Some(node_index) = bone_lookup.get(bone) else {
            eprintln!("Warning: bone id {} not found in skeleton.", bone);
            continue;
        };

        if root_motion_applied && *node_index == root_node {
            continue;
        }

        let times_accessor = create_scalar_accessor(root, times);
        let values_accessor = create_vec3_accessor(root, values, None, None);

        let sampler = Index::push(
            &mut animation.samplers,
            Sampler {
                extensions: None,
                extras: None,
                input: times_accessor,
                interpolation: Valid(Interpolation::Linear),
                output: values_accessor,
            },
        );

        animation.channels.push(Channel {
            sampler,
            target: Target {
                extensions: None,
                extras: None,
                node: *node_index,
                path: Valid(Property::Translation),
            },
            extensions: None,
            extras: None,
        });
    }

    for (bone, (times, values)) in rotations.iter() {
        let Some(node_index) = bone_lookup.get(bone) else {
            eprintln!("Warning: bone id {} not found in skeleton.", bone);
            continue;
        };

        let times_accessor = create_scalar_accessor(root, times);
        let values_accessor = create_vec4_accessor(root, values);

        let sampler = Index::push(
            &mut animation.samplers,
            Sampler {
                extensions: None,
                extras: None,
                input: times_accessor,
                interpolation: Valid(Interpolation::Linear),
                output: values_accessor,
            },
        );

        animation.channels.push(Channel {
            sampler,
            target: Target {
                extensions: None,
                extras: None,
                node: *node_index,
                path: Valid(Property::Rotation),
            },
            extensions: None,
            extras: None,
        });
    }

    root.push(animation);
}

fn convert_position(v: Vec3, scale: f32) -> Vec3 {
    let m = CONVERT * Mat4::from_scale(Vec3::splat(scale));
    m.project_point3(v)
}

fn resolve_bone_id(motion: &bmf::Motion, raw_id: u32) -> u32 {
    motion
        .bone_ids
        .get(raw_id as usize)
        .copied()
        .unwrap_or(raw_id)
}

fn select_root_motion_bone(
    scene: &smf::Model,
    motion: &bmf::Motion,
    override_id: Option<u32>,
) -> Option<u32> {
    if let Some(id) = override_id {
        return Some(id);
    }

    let root_name = scene.nodes.first()?.name.as_str();
    let root_children = scene
        .nodes
        .iter()
        .filter(|n| n.parent_name == root_name)
        .map(|n| n.tree_id)
        .collect::<Vec<_>>();

    for child in root_children.iter() {
        if motion_has_translation(motion, *child) {
            return Some(*child);
        }
    }

    let root_id = scene.nodes.first().map(|n| n.tree_id)?;
    if motion_has_translation(motion, root_id) {
        return Some(root_id);
    }

    None
}

fn motion_has_translation(motion: &bmf::Motion, bone_id: u32) -> bool {
    motion
        .key_frames
        .iter()
        .flat_map(|kf| kf.bones.iter())
        .any(|bone| bone.position.is_some() && resolve_bone_id(motion, bone.bone_id) == bone_id)
}

fn convert_rotation(q: Quat) -> Quat {
    convert_rotation_basis(q, CONVERT)
}

fn convert_position_bmf(v: Vec3, scale: f32) -> Vec3 {
    let m = CONVERT_BMF * Mat4::from_scale(Vec3::splat(scale));
    m.project_point3(v)
}

fn convert_rotation_bmf(q: Quat) -> Quat {
    convert_rotation_basis(q, CONVERT_BMF)
}

fn convert_rotation_basis(q: Quat, basis: Mat4) -> Quat {
    let basis_inv = basis.inverse();
    let rot = Mat4::from_quat(q);
    let converted = basis * rot * basis_inv;
    Quat::from_mat4(&converted)
}

fn resolve_material(
    root: &mut Root,
    mesh: &smf::Mesh,
    texture_roots: &[PathBuf],
    materials: &mut HashMap<String, Index<json::Material>>,
) -> Index<json::Material> {
    if let Some(existing) = materials.get(&mesh.texture_name) {
        return *existing;
    }

    let image_uri = if let Some(image_path) = texture_roots
        .iter()
        .find_map(|root| find_image_path(root, &mesh.texture_name))
    {
        image_to_buffer(image_path).expect("Could not embed image.")
    } else {
        eprintln!("Warning: Could not find image: {}", mesh.texture_name);
        PathBuf::from(&mesh.texture_name)
            .to_str()
            .unwrap()
            .to_owned()
    };

    let image_index = root.push(json::Image {
        buffer_view: None,
        mime_type: None,
        name: Some(mesh.name.clone()),
        uri: Some(image_uri),
        extensions: None,
        extras: None,
    });

    let texture_index = root.push(json::Texture {
        name: None,
        sampler: None,
        source: image_index,
        extensions: None,
        extras: None,
    });

    let material_index = root.push(json::Material {
        alpha_cutoff: None,
        alpha_mode: Valid(material::AlphaMode::Opaque),
        double_sided: true,
        name: None,
        pbr_metallic_roughness: material::PbrMetallicRoughness {
            base_color_factor: material::PbrBaseColorFactor([1.0, 1.0, 1.0, 1.0]),
            base_color_texture: Some(json::texture::Info {
                index: texture_index,
                tex_coord: 0,
                extensions: None,
                extras: None,
            }),
            metallic_factor: material::StrengthFactor(0.0),
            roughness_factor: material::StrengthFactor(1.0),
            metallic_roughness_texture: None,
            extensions: None,
            extras: None,
        },
        normal_texture: None,
        occlusion_texture: None,
        emissive_texture: None,
        emissive_factor: material::EmissiveFactor([0.0, 0.0, 0.0]),
        extensions: None,
        extras: None,
    });

    materials.insert(mesh.texture_name.clone(), material_index);

    material_index
}

fn create_scalar_accessor(root: &mut Root, data: &[f32]) -> Index<Accessor> {
    let buffer = root.push(create_buffer(data));
    let view = root.push(buffer::View {
        buffer,
        byte_length: std::mem::size_of_val(data).into(),
        byte_offset: None,
        byte_stride: None,
        name: None,
        target: None,
        extensions: None,
        extras: None,
    });

    root.push(Accessor {
        buffer_view: Some(view),
        byte_offset: None,
        count: data.len().into(),
        component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::F32)),
        extensions: None,
        extras: None,
        type_: Valid(accessor::Type::Scalar),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    })
}

fn create_vec2_accessor(root: &mut Root, data: &[[f32; 2]]) -> Index<Accessor> {
    let buffer = root.push(create_buffer(data));
    let view = root.push(buffer::View {
        buffer,
        byte_length: std::mem::size_of_val(data).into(),
        byte_offset: None,
        byte_stride: None,
        name: None,
        target: Some(Valid(buffer::Target::ArrayBuffer)),
        extensions: None,
        extras: None,
    });

    root.push(Accessor {
        buffer_view: Some(view),
        byte_offset: None,
        count: data.len().into(),
        component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::F32)),
        extensions: None,
        extras: None,
        type_: Valid(accessor::Type::Vec2),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    })
}

fn create_vec3_accessor(
    root: &mut Root,
    data: &[[f32; 3]],
    min: Option<[f32; 3]>,
    max: Option<[f32; 3]>,
) -> Index<Accessor> {
    let buffer = root.push(create_buffer(data));
    let view = root.push(buffer::View {
        buffer,
        byte_length: std::mem::size_of_val(data).into(),
        byte_offset: None,
        byte_stride: None,
        name: None,
        target: Some(Valid(buffer::Target::ArrayBuffer)),
        extensions: None,
        extras: None,
    });

    root.push(Accessor {
        buffer_view: Some(view),
        byte_offset: None,
        count: data.len().into(),
        component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::F32)),
        extensions: None,
        extras: None,
        type_: Valid(accessor::Type::Vec3),
        min: min.map(|v| json::Value::from(Vec::from(v))),
        max: max.map(|v| json::Value::from(Vec::from(v))),
        name: None,
        normalized: false,
        sparse: None,
    })
}

fn create_vec4_accessor(root: &mut Root, data: &[[f32; 4]]) -> Index<Accessor> {
    let buffer = root.push(create_buffer(data));
    let view = root.push(buffer::View {
        buffer,
        byte_length: std::mem::size_of_val(data).into(),
        byte_offset: None,
        byte_stride: None,
        name: None,
        target: Some(Valid(buffer::Target::ArrayBuffer)),
        extensions: None,
        extras: None,
    });

    root.push(Accessor {
        buffer_view: Some(view),
        byte_offset: None,
        count: data.len().into(),
        component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::F32)),
        extensions: None,
        extras: None,
        type_: Valid(accessor::Type::Vec4),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    })
}

fn create_u16x4_accessor(root: &mut Root, data: &[[u16; 4]]) -> Index<Accessor> {
    let buffer = root.push(create_buffer(data));
    let view = root.push(buffer::View {
        buffer,
        byte_length: std::mem::size_of_val(data).into(),
        byte_offset: None,
        byte_stride: None,
        name: None,
        target: Some(Valid(buffer::Target::ArrayBuffer)),
        extensions: None,
        extras: None,
    });

    root.push(Accessor {
        buffer_view: Some(view),
        byte_offset: None,
        count: data.len().into(),
        component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::U16)),
        extensions: None,
        extras: None,
        type_: Valid(accessor::Type::Vec4),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    })
}

fn create_indices_accessor(root: &mut Root, data: &[u32]) -> Index<Accessor> {
    let buffer = root.push(create_buffer(data));
    let view = root.push(buffer::View {
        buffer,
        byte_length: std::mem::size_of_val(data).into(),
        byte_offset: None,
        byte_stride: None,
        name: None,
        target: Some(Valid(buffer::Target::ElementArrayBuffer)),
        extensions: None,
        extras: None,
    });

    root.push(Accessor {
        buffer_view: Some(view),
        byte_offset: None,
        count: data.len().into(),
        component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::U32)),
        extensions: None,
        extras: None,
        type_: Valid(accessor::Type::Scalar),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    })
}

fn create_mat4_accessor(root: &mut Root, data: &[[f32; 16]]) -> Index<Accessor> {
    let buffer = root.push(create_buffer(data));
    let view = root.push(buffer::View {
        buffer,
        byte_length: std::mem::size_of_val(data).into(),
        byte_offset: None,
        byte_stride: None,
        name: None,
        target: None,
        extensions: None,
        extras: None,
    });

    root.push(Accessor {
        buffer_view: Some(view),
        byte_offset: None,
        count: data.len().into(),
        component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::F32)),
        extensions: None,
        extras: None,
        type_: Valid(accessor::Type::Mat4),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    })
}

fn create_buffer<T>(data: &[T]) -> Buffer
where
    T: bytemuck::Pod,
{
    let mut bytes = bytemuck::cast_slice(data).to_vec();
    while bytes.len() % 4 != 0 {
        bytes.push(0);
    }

    let data_uri = create_data_uri(&bytes, "application/octet-stream");

    Buffer {
        byte_length: json::validation::USize64::from(bytes.len()),
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

fn find_image_path(root: impl AsRef<Path>, name: &str) -> Option<PathBuf> {
    walkdir::WalkDir::new(root.as_ref())
        .into_iter()
        .filter_map(Result::ok)
        .find(|e| e.file_name().eq_ignore_ascii_case(name))
        .map(|e| e.into_path())
}

fn texture_search_roots(smf_path: &Path, out_path: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(textures_root) = find_textures_root(smf_path) {
        roots.push(textures_root);
    }

    if let Some(out_parent) = out_path.parent() {
        if !roots.iter().any(|root| root == out_parent) {
            roots.push(out_parent.to_path_buf());
        }
    }

    roots
}

fn find_textures_root(smf_path: &Path) -> Option<PathBuf> {
    let mut current = smf_path.parent();
    while let Some(dir) = current {
        let is_data = dir
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.eq_ignore_ascii_case("data"))
            .unwrap_or(false);
        if is_data {
            let textures = dir.join("textures");
            if textures.is_dir() {
                return Some(textures);
            }
        }
        current = dir.parent();
    }

    None
}

fn image_to_buffer(image_path: impl AsRef<Path>) -> std::io::Result<String> {
    let file = std::fs::File::open(image_path.as_ref())?;
    let mut reader = std::io::BufReader::new(file);

    let ext = image_path.as_ref().extension().unwrap();
    if !ext.eq_ignore_ascii_case("bmp") {
        panic!("Invalid image format!");
    }

    let bmp_image = shadow_company_tools::images::load_bmp_file(&mut reader, false)
        .expect("Could not open .bmp file");

    let raw_path = image_path.as_ref().with_extension("raw");
    let png = if raw_path.exists() {
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

fn bounding_coords(points: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN];

    for point in points {
        for i in 0..3 {
            min[i] = f32::min(min[i], point[i]);
            max[i] = f32::max(max[i], point[i]);
        }
    }

    (min, max)
}

fn output_path_for_mode(smf_path: &Path, bmf_path: Option<&Path>, mode: ExportMode) -> PathBuf {
    match mode {
        ExportMode::Anim => bmf_path.unwrap_or(smf_path).with_extension("gltf"),
        ExportMode::Mesh | ExportMode::Full => smf_path.with_extension("gltf"),
    }
}

fn missing_bmf_exit(mode: ExportMode) -> PathBuf {
    eprintln!(
        "Missing .bmf file. A motion file is required for --mode {:?}.",
        mode
    );
    std::process::exit(2);
}
