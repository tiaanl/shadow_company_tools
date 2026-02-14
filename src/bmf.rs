use bitflags::bitflags;
use glam::{Quat, Vec3};

use crate::io;

#[derive(Clone, Debug)]
pub struct Bone {
    pub bone_id: u32,
    pub time: u32,
    pub rotation: Option<Quat>,
    pub position: Option<Vec3>,
}

#[derive(Debug)]
pub struct KeyFrame {
    pub frame: u32,
    // Likely linear velocity extraction / root motion vector for this frame.
    pub lve: Vec3,
    // Unknown keyframe-local pointer/id-like value.
    pub keyframe_data_ref: u32,
    // Number of bones in this keyframe block.
    pub bone_count: u32,
    // Reserved/unused in observed files.
    pub reserved_0: u32,
    // Reserved/unused in observed files.
    pub reserved_1: u32,
    pub bones: Vec<Bone>,
}

// Known state values:
//   1 = stand
//   2 = crouch
//   3 = prone
//   4 = on_back
//   5 = sit
//   6 = scuba

#[derive(Debug)]
pub struct Motion {
    pub name: String,
    pub name_hash: u32,
    pub key_frame_count: u32,
    pub last_frame: u32,
    pub playback_rate: u32,
    pub max_bones_per_frame: u32,
    pub from_state: u32,
    pub to_state: u32,

    pub key_frames: Vec<KeyFrame>,

    pub bone_ids: Vec<u32>,
}

bitflags! {
    struct KeyFrameFlags : u8 {
        const HAS_ROTATION = 1 << 0;
        const HAS_POSITION = 1 << 1;
    }
}

impl Motion {
    pub fn read(r: &mut impl io::Reader) -> std::io::Result<Self> {
        let name = r.read_fixed_string(0x80)?;

        let name_hash = r.read_u32()?;
        let _ = r.read_u32()?; // Unknown pointer/id-like field.
        let key_frame_count = r.read_u32()?;
        let _ = r.read_u32()?; // Always 0.
        let last_frame = r.read_u32()?;
        let playback_rate = r.read_u32()?; // Usually 10, sometimes 30.
        let _ = r.read_u32()?; // Unknown timing constant (usually 480, sometimes 160).
        let _ = r.read_u32()?; // Unknown timing constant (usually 100, sometimes 33).
        let _ = r.read_u32()?; // Unknown pointer/id-like field.
        let max_bones_per_frame = r.read_u32()?;
        let from_state = r.read_u32()?;
        let to_state = r.read_u32()?;

        let mut key_frames = vec![];
        key_frames.reserve_exact(key_frame_count as usize);

        for _ in 0..key_frame_count {
            let lve = r.read_vec3()?;

            let time = r.read_u32()?;

            let keyframe_data_ref = r.read_u32()?;
            let bone_count = r.read_u32()?;
            let reserved_0 = r.read_u32()?;
            let reserved_1 = r.read_u32()?;

            let mut bones = Vec::with_capacity(bone_count as usize);
            for _ in 0..bone_count {
                let bone_time = r.read_u32()?;
                assert!(
                    time == bone_time,
                    "Keyframe time and bone time do not match!"
                );
                let bone_index = r.read_u32()?;
                let flags = KeyFrameFlags::from_bits_truncate(r.read_u8()?);

                let rotation = if flags.contains(KeyFrameFlags::HAS_ROTATION) {
                    let mut rotation = Quat::IDENTITY;
                    rotation.w = r.read_f32()?;
                    rotation.x = r.read_f32()?;
                    rotation.y = r.read_f32()?;
                    rotation.z = r.read_f32()?;

                    Some(rotation)
                } else {
                    None
                };

                let position = if flags.contains(KeyFrameFlags::HAS_POSITION) {
                    Some(r.read_vec3()?)
                } else {
                    None
                };

                bones.push(Bone {
                    time: bone_time,
                    bone_id: bone_index,
                    rotation,
                    position,
                });
            }

            key_frames.push(KeyFrame {
                frame: time,
                lve,
                keyframe_data_ref,
                bone_count,
                reserved_0,
                reserved_1,
                bones,
            });
        }

        let mut bone_indices = vec![];
        bone_indices.reserve_exact(max_bones_per_frame as usize);
        for _ in 0..max_bones_per_frame {
            let bone_index = r.read_u32()?;
            bone_indices.push(bone_index);
        }

        Ok(Self {
            name,
            key_frames,
            bone_ids: bone_indices,

            name_hash,
            key_frame_count,
            last_frame,
            playback_rate,
            max_bones_per_frame,
            from_state,
            to_state,
        })
    }
}
