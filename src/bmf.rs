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
    // Number of bones in this keyframe block.
    pub bone_count: u32,
    // Reserved/unused in observed files.
    pub reserved_0: u32,
    // Reserved/unused in observed files.
    pub reserved_1: u32,
    pub bones: Vec<Bone>,
}

bitflags! {
    #[derive(Debug)]
    pub struct MotionFlags: u32 {
        const DECLARE_Z_IND_MOTION = 1 << 0;
        const DECLARE_NO_LVE_MOTION = 1 << 1;
        const DECLARE_SKIP_LAST_FRAME = 1 << 2;
        const DECLARE_SPED_MOTION = 1 << 3;
    }
}

// Known state values:
//   0 = none (placeholder)
//   1 = stand
//   2 = crouch
//   3 = prone
//   4 = on_back
//   5 = sit
//   6 = scuba

#[derive(Debug)]
pub struct Motion {
    pub name: String,
    pub flags: MotionFlags,
    pub hash: u32,
    pub key_frame_count: u32,
    pub last_frame: u32,
    pub ticks_per_frame: u32,
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
        let name = r.read_fixed_string(124)?;
        let flags = MotionFlags::from_bits_retain(r.read_u32()?);
        let hash = r.read_u32()?;
        let _ = r.read_u32()?; // Unknown pointer/id-like field.
        let key_frame_count = r.read_u32()?;
        let _ = r.read_u32()?; // Always 0.
        let last_frame = r.read_u32()?;
        let ticks_per_frame = r.read_u32()?; // Usually 10, sometimes 30.
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

            let frame = r.read_u32()?;

            // This is a pointer as is serialized directly like that, so we skip it.
            let _bones_pointer = r.read_u32()?;

            let bone_count = r.read_u32()?;
            let reserved_0 = r.read_u32()?;
            let reserved_1 = r.read_u32()?;

            let mut bones = Vec::with_capacity(bone_count as usize);
            for _ in 0..bone_count {
                let bone_frame = r.read_u32()?;
                assert!(
                    frame == bone_frame,
                    "Keyframe time and bone time do not match!"
                );
                let tree_id = r.read_u32()?;
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
                    let mut position = Vec3::ZERO;
                    position.x = r.read_f32()?;
                    position.y = r.read_f32()?;
                    position.z = r.read_f32()?;
                    Some(position)
                } else {
                    None
                };

                bones.push(Bone {
                    time: bone_frame,
                    bone_id: tree_id,
                    rotation,
                    position,
                });
            }

            key_frames.push(KeyFrame {
                frame,
                lve,
                bone_count,
                reserved_0,
                reserved_1,
                bones,
            });
        }

        let mut bone_ids = vec![];
        bone_ids.reserve_exact(max_bones_per_frame as usize);
        for _ in 0..max_bones_per_frame {
            let bone_index = r.read_u32()?;
            bone_ids.push(bone_index);
        }

        Ok(Self {
            name,
            flags,
            hash,
            key_frame_count,
            last_frame,
            ticks_per_frame,
            max_bones_per_frame,
            from_state,
            to_state,
            key_frames,
            bone_ids,
        })
    }
}
