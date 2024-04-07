use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Result};

use glam::{Quat, Vec3};

pub fn read_vec3(reader: &mut impl Read) -> Result<Vec3> {
    let x = reader.read_f32::<LittleEndian>()?;
    let y = reader.read_f32::<LittleEndian>()?;
    let z = reader.read_f32::<LittleEndian>()?;
    Ok(Vec3::new(x, y, z))
}

pub fn read_quat(reader: &mut impl Read) -> Result<Quat> {
    let w = reader.read_f32::<LittleEndian>()?;
    let x = reader.read_f32::<LittleEndian>()?;
    let y = reader.read_f32::<LittleEndian>()?;
    let z = reader.read_f32::<LittleEndian>()?;
    Ok(Quat::from_xyzw(x, y, z, w))
}
