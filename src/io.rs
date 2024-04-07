use byteorder::{LittleEndian, ReadBytesExt};
use glam::{Quat, Vec3};
use std::io::Result;

pub trait GlamReadExt: Sized {
    fn read(reader: &mut impl std::io::Read) -> Result<Self>;
}

impl GlamReadExt for Vec3 {
    fn read(reader: &mut impl std::io::Read) -> Result<Self> {
        let x = reader.read_f32::<LittleEndian>()?;
        let y = reader.read_f32::<LittleEndian>()?;
        let z = reader.read_f32::<LittleEndian>()?;
        Ok(Self::new(x, y, z))
    }
}

impl GlamReadExt for Quat {
    fn read(reader: &mut impl std::io::Read) -> Result<Self> {
        let w = reader.read_f32::<LittleEndian>()?;
        let x = reader.read_f32::<LittleEndian>()?;
        let y = reader.read_f32::<LittleEndian>()?;
        let z = reader.read_f32::<LittleEndian>()?;
        Ok(Quat::from_xyzw(x, y, z, w))
    }
}
