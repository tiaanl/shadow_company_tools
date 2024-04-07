use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use glam::{Quat, Vec3};

use crate::{common::read_fixed_string, io::GlamReadExt};

#[derive(Clone, Debug)]
pub struct Bone {
    pub bone_index: u32,
    pub time: u32,
    pub rotation: Option<Quat>,
    pub position: Option<Vec3>,
}

#[derive(Debug)]
pub struct KeyFrame {
    pub time: u32,
    pub bones: Vec<Bone>,
}

#[derive(Debug)]
pub struct Motion {
    pub name: String,
    pub key_frames: Vec<KeyFrame>,
    pub bone_indices: Vec<u32>,
}

bitflags! {
    struct KeyFrameFlags : u8 {
        const HAS_ROTATION = 1 << 0;
        const HAS_POSITION = 1 << 1;
    }
}

impl Motion {
    pub fn read<R>(c: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read + std::io::Seek,
    {
        let name = read_fixed_string(c, 0x80);

        let _ = c.read_u32::<LittleEndian>()?;
        let _ = c.read_u32::<LittleEndian>()?;

        let count = c.read_u32::<LittleEndian>()?;

        c.seek(std::io::SeekFrom::Start(0xB0))?;

        let mut max = 0;

        let mut key_frames = vec![];
        key_frames.reserve_exact(count as usize);

        for _ in 0..count {
            let _ = c.read_u32::<LittleEndian>()?; // always 0
            let _ = c.read_u32::<LittleEndian>()?; // always 0
            let _ = c.read_u32::<LittleEndian>()?; // always 0
            let time = c.read_u32::<LittleEndian>()?;

            // 0082b004, 0082b38c, 0082b714, 0082ba9c, 0082be24, 0082c1ac, 0082c534, 0082c8bc, 0082cc44, 0082cfcc, 0082d354
            // 009FCAA4, 009FC704, 009FC364, 009FDC84, 009FD8E4, 009FD544, 009FD1A4, 009FEC84, 009FE8E4
            let _ = c.read_u32::<LittleEndian>()?;

            let bone_count = c.read_u32::<LittleEndian>()?;
            max = max.max(bone_count);
            let _ = c.read_u32::<LittleEndian>()?; // always 0
            let _ = c.read_u32::<LittleEndian>()?; // always 0

            let mut bones = Vec::with_capacity(bone_count as usize);
            for _ in 0..bone_count {
                let bone_time = c.read_u32::<LittleEndian>()?;
                assert!(
                    time == bone_time,
                    "Keyframe time and bone time do not match!"
                );
                let bone_index = c.read_u32::<LittleEndian>()?;
                let flags = KeyFrameFlags::from_bits_truncate(c.read_u8()?);

                let rotation = if flags.contains(KeyFrameFlags::HAS_ROTATION) {
                    let mut rotation = Quat::IDENTITY;
                    rotation.w = c.read_f32::<LittleEndian>()?;
                    rotation.x = c.read_f32::<LittleEndian>()?;
                    rotation.y = c.read_f32::<LittleEndian>()?;
                    rotation.z = c.read_f32::<LittleEndian>()?;

                    Some(rotation)
                } else {
                    None
                };

                let position = if flags.contains(KeyFrameFlags::HAS_POSITION) {
                    Some(Vec3::read(c)?)
                } else {
                    None
                };

                bones.push(Bone {
                    time: bone_time,
                    bone_index,
                    rotation,
                    position,
                });
            }

            key_frames.push(KeyFrame { time, bones });
        }

        let mut bone_indices = vec![];
        bone_indices.reserve_exact(max as usize);
        for _ in 0..max {
            let bone_index = c.read_u32::<LittleEndian>()?;
            bone_indices.push(bone_index);
        }

        Ok(Self {
            name,
            key_frames,
            bone_indices,
        })
    }
}
