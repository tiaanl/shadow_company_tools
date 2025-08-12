use core::f32;
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use bytemuck::NoUninit;
use clap::Parser;
use gltf_json::{
    accessor::{self, ComponentType, GenericComponentType},
    animation::{Channel, Interpolation, Property, Sampler, Target},
    buffer::{Stride, View},
    mesh::{Primitive, Semantic},
    validation::{Checked, USize64},
    Accessor, Animation, Buffer, Index, Mesh, Node, Root, Scene,
};
use shadow_company_tools::{bmf, smf, Quat, Vec3, Vec4};

#[derive(Parser)]
struct Opts {
    /// Model file containing the skeleton for the motion.
    smf_path: PathBuf,
    /// Motion file.
    bmf_path: PathBuf,
}

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
}

#[derive(Clone, Copy, Debug, NoUninit)]
#[repr(C)]
struct TimedTranslation {
    time: u32,
    translation: Vec3,
}

#[derive(Clone, Copy, Debug, NoUninit)]
#[repr(C)]
struct TimedRotation {
    time: u32,
    _padding: [f32; 3],
    rotation: Quat,
}

impl From<(Vec3, Vec3)> for Vertex {
    fn from(value: (Vec3, Vec3)) -> Self {
        Self {
            position: value.0,
            normal: value.1,
        }
    }
}

fn main() {
    let opts = Opts::parse();

    let smf = smf::Model::read(
        &mut std::fs::File::open(&opts.smf_path).expect("Could not open .smf file."),
    )
    .expect("Could not parse .smf file.");

    let bmf = bmf::Motion::read(
        &mut std::fs::File::open(&opts.bmf_path).expect("Could not open .bmf file."),
    )
    .expect("Could not parse .bmf file.");

    let mut bone_lookup = HashMap::new();

    let mut root = Root::default();

    assert_eq!(smf.nodes[0].parent_name, "<root>");
    add_node(&mut root, smf.nodes.as_slice(), 0, &mut bone_lookup);

    add_animation(&mut root, &bmf, &bone_lookup);

    let str = gltf_json::serialize::to_string_pretty(&root).unwrap();
    println!("{}", str);

    gltf_json::serialize::to_writer_pretty(&mut std::fs::File::create("test.gltf").unwrap(), &root)
        .unwrap();
}

fn add_node(
    root: &mut Root,
    nodes: &[smf::Node],
    node_index: usize,
    bone_lookup: &mut HashMap<u32, Index<Node>>,
) -> Index<Node> {
    let node = &nodes[node_index];

    let children = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.parent_name == node.name)
        .map(|(i, _)| add_node(root, nodes, i, bone_lookup))
        .collect();

    let index = root.push(Node {
        camera: None,
        children: Some(children),
        extensions: None,
        extras: None,
        matrix: None,
        mesh: None,
        name: Some(node.name.clone()),
        rotation: None,
        scale: None,
        translation: Some(smf::CONVERT.project_point3(node.position).to_array()),
        skin: None,
        weights: None,
    });

    bone_lookup.insert(node.tree_id, index);

    index
}

fn add_animation(root: &mut Root, motion: &bmf::Motion, bone_lookup: &HashMap<u32, Index<Node>>) {
    let mut animation = Animation {
        extensions: None,
        extras: None,
        channels: vec![],
        name: Some(motion.name.clone()),
        samplers: vec![],
    };

    let mut translations = HashMap::<u32, Vec<TimedTranslation>>::new();
    let mut rotations = HashMap::<u32, Vec<TimedRotation>>::new();

    for key_frame in motion.key_frames.iter() {
        for bone in key_frame.bones.iter() {
            if let Some(translation) = bone.position {
                let translations = translations.entry(bone.bone_index).or_default();
                translations.push(TimedTranslation {
                    time: bone.time,
                    translation,
                });
            }

            if let Some(rotation) = bone.rotation {
                let rotations = rotations.entry(bone.bone_index).or_default();

                // Convert the rotation to right-handed y-up.
                let rotation_z_to_y = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                let transformed_quaternion = rotation_z_to_y * rotation;
                let rotation = Quat::from_xyzw(
                    -transformed_quaternion.x,
                    transformed_quaternion.y,
                    transformed_quaternion.z,
                    transformed_quaternion.w,
                );

                rotations.push(TimedRotation {
                    time: bone.time,
                    _padding: [0.0; 3],
                    rotation,
                });
            }
        }
    }

    if translations.is_empty() && rotations.is_empty() {
        return;
    }

    // println!("{:#?}", translations);
    // println!("{:#?}", rotations);

    // Translations
    for (bone, translations) in translations.iter() {
        let buffer = root.push(create_buffer(translations.as_slice()));
        let times_view = root.push(View {
            buffer,
            byte_length: std::mem::size_of_val(translations.as_slice()).into(),
            byte_offset: None,
            byte_stride: Some(Stride(std::mem::size_of::<TimedTranslation>())),
            name: None,
            target: None,
            extensions: None,
            extras: None,
        });

        let times_accessor = root.push(Accessor {
            buffer_view: Some(times_view),
            byte_offset: None,
            count: translations.len().into(),
            component_type: Checked::Valid(GenericComponentType(ComponentType::U32)),
            extensions: None,
            extras: None,
            type_: Checked::Valid(accessor::Type::Scalar),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        let translations_view = root.push(View {
            buffer,
            byte_length: std::mem::size_of_val(translations.as_slice()).into(),
            byte_offset: Some(std::mem::size_of::<f32>().into()),
            byte_stride: Some(Stride(std::mem::size_of::<TimedTranslation>())),
            name: None,
            target: None,
            extensions: None,
            extras: None,
        });

        let translations_accessor = root.push(Accessor {
            buffer_view: Some(translations_view),
            byte_offset: None,
            count: translations.len().into(),
            component_type: Checked::Valid(GenericComponentType(ComponentType::F32)),
            extensions: None,
            extras: None,
            type_: Checked::Valid(accessor::Type::Vec3),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        let translations_sampler = Index::push(
            &mut animation.samplers,
            Sampler {
                extensions: None,
                extras: None,
                input: times_accessor,
                interpolation: Checked::Valid(Interpolation::Linear),
                output: translations_accessor,
            },
        );

        animation.channels.push(Channel {
            sampler: translations_sampler,
            target: Target {
                extensions: None,
                extras: None,
                node: *bone_lookup.get(bone).unwrap(),
                path: Checked::Valid(Property::Translation),
            },
            extensions: None,
            extras: None,
        });
    }

    // Rotations
    for (bone, rotations) in rotations.iter() {
        let buffer = root.push(create_buffer(rotations.as_slice()));
        let times_view = root.push(View {
            buffer,
            byte_length: std::mem::size_of_val(rotations.as_slice()).into(),
            byte_offset: None,
            byte_stride: Some(Stride(std::mem::size_of::<TimedRotation>())),
            name: None,
            target: None,
            extensions: None,
            extras: None,
        });

        let times_accessor = root.push(Accessor {
            buffer_view: Some(times_view),
            byte_offset: None,
            count: rotations.len().into(),
            component_type: Checked::Valid(GenericComponentType(ComponentType::U32)),
            extensions: None,
            extras: None,
            type_: Checked::Valid(accessor::Type::Scalar),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        let rotations_view = root.push(View {
            buffer,
            byte_length: std::mem::size_of_val(rotations.as_slice()).into(),
            byte_offset: Some(std::mem::size_of::<Vec4>().into()),
            byte_stride: Some(Stride(std::mem::size_of::<TimedRotation>())),
            name: None,
            target: None,
            extensions: None,
            extras: None,
        });

        let rotations_accessor = root.push(Accessor {
            buffer_view: Some(rotations_view),
            byte_offset: None,
            count: rotations.len().into(),
            component_type: Checked::Valid(GenericComponentType(ComponentType::F32)),
            extensions: None,
            extras: None,
            type_: Checked::Valid(accessor::Type::Vec4),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        let rotations_sampler = Index::push(
            &mut animation.samplers,
            Sampler {
                extensions: None,
                extras: None,
                input: times_accessor,
                interpolation: Checked::Valid(Interpolation::Linear),
                output: rotations_accessor,
            },
        );

        animation.channels.push(Channel {
            sampler: rotations_sampler,
            target: Target {
                extensions: None,
                extras: None,
                node: *bone_lookup.get(bone).unwrap(),
                path: Checked::Valid(Property::Rotation),
            },
            extensions: None,
            extras: None,
        });
    }

    root.push(animation);
}

fn create_buffer<T>(buffer: &[T]) -> Buffer
where
    T: NoUninit,
{
    let byte_length = std::mem::size_of_val(buffer);

    let data_uri = create_data_uri(bytemuck::cast_slice(buffer), "application/octet-stream");

    Buffer {
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
