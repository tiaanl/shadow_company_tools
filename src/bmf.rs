use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
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
    pub bones: Vec<Bone>,
}

#[derive(Debug)]
pub struct Motion {
    pub name: String,
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

        let _ = r.read_u32::<LittleEndian>()?;
        let _ = r.read_u32::<LittleEndian>()?;

        let count = r.read_u32::<LittleEndian>()?;

        r.seek(std::io::SeekFrom::Start(0xB0))?;

        let mut max = 0;

        let mut key_frames = vec![];
        key_frames.reserve_exact(count as usize);

        for _ in 0..count {
            let _ = r.read_u32::<LittleEndian>()?; // always 0
            let _ = r.read_u32::<LittleEndian>()?; // always 0
            let _ = r.read_u32::<LittleEndian>()?; // always 0
            let time = r.read_u32::<LittleEndian>()?;

            // 0082b004, 0082b38c, 0082b714, 0082ba9c, 0082be24, 0082c1ac, 0082c534, 0082c8bc, 0082cc44, 0082cfcc, 0082d354
            // 009FCAA4, 009FC704, 009FC364, 009FDC84, 009FD8E4, 009FD544, 009FD1A4, 009FEC84, 009FE8E4
            let _ = r.read_u32::<LittleEndian>()?;

            let bone_count = r.read_u32::<LittleEndian>()?;
            max = max.max(bone_count);
            let _ = r.read_u32::<LittleEndian>()?; // always 0
            let _ = r.read_u32::<LittleEndian>()?; // always 0

            let mut bones = Vec::with_capacity(bone_count as usize);
            for _ in 0..bone_count {
                let bone_time = r.read_u32::<LittleEndian>()?;
                assert!(
                    time == bone_time,
                    "Keyframe time and bone time do not match!"
                );
                let bone_index = r.read_u32::<LittleEndian>()?;
                let flags = KeyFrameFlags::from_bits_truncate(r.read_u8()?);

                let rotation = if flags.contains(KeyFrameFlags::HAS_ROTATION) {
                    let mut rotation = Quat::IDENTITY;
                    rotation.w = r.read_f32::<LittleEndian>()?;
                    rotation.x = r.read_f32::<LittleEndian>()?;
                    rotation.y = r.read_f32::<LittleEndian>()?;
                    rotation.z = r.read_f32::<LittleEndian>()?;

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

            key_frames.push(KeyFrame { frame: time, bones });
        }

        let mut bone_indices = vec![];
        bone_indices.reserve_exact(max as usize);
        for _ in 0..max {
            let bone_index = r.read_u32::<LittleEndian>()?;
            bone_indices.push(bone_index);
        }

        Ok(Self {
            name,
            key_frames,
            bone_ids: bone_indices,
        })
    }
}
