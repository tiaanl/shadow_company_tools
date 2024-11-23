use byteorder::{LittleEndian as LE, ReadBytesExt};
use glam::{Quat, Vec2, Vec3};

use crate::io::Reader;

/// A container for an single model.
#[derive(Debug)]
pub struct Model {
    pub name: String,
    pub scale: Vec3,
    pub nodes: Vec<Node>,
}

impl Model {
    pub fn read(r: &mut impl Reader) -> std::io::Result<Self> {
        let _ = r.skip_sinister_header()?;

        let version_string = r.read_fixed_string(16)?;
        let Some(smf_version) = smf_version(&version_string) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid SMF version.",
            ));
        };

        let name = r.read_fixed_string(128)?;

        let scale = r.read_vec3()?;

        let _ = r.read_f32::<LE>()?; // usually == 1.0
        let _ = r.read_u32::<LE>()?; // usually == 1

        let node_count = r.read_u32::<LE>()?;

        let mut nodes = Vec::with_capacity(node_count as usize);
        for _ in 0..node_count {
            nodes.push(Node::read(r, smf_version)?);
        }

        Ok(Self { name, scale, nodes })
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub texture_name: String,
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
}

#[derive(Debug)]
pub struct Face {
    pub index: u32,
    pub indices: [u32; 3],
}

impl Face {
    fn read(r: &mut impl Reader) -> std::io::Result<Self> {
        let index = r.read_u32::<LE>()?;
        let i0 = r.read_u32::<LE>()?;
        let i1 = r.read_u32::<LE>()?;
        let i2 = r.read_u32::<LE>()?;

        Ok(Face {
            index,
            indices: [i0, i1, i2],
        })
    }
}

impl Mesh {
    fn read(r: &mut impl Reader) -> std::io::Result<Self> {
        let name = r.read_fixed_string(128)?;
        let texture_name = r.read_fixed_string(128)?;

        let vertex_count = r.read_u32::<LE>()?;
        let face_count = r.read_u32::<LE>()?;

        let mut vertices = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            vertices.push(Vertex::read(r)?);
        }

        let mut faces = Vec::with_capacity(face_count as usize);
        for _ in 0..face_count {
            faces.push(Face::read(r)?);
        }

        Ok(Mesh {
            name,
            texture_name,
            vertices,
            faces,
        })
    }
}

#[derive(Debug)]
pub struct Vertex {
    pub index: u32,
    pub position: Vec3,
    pub tex_coord: Vec2,
    pub normal: Vec3,
}

impl Vertex {
    fn read(r: &mut impl Reader) -> std::io::Result<Self> {
        let index = r.read_u32::<LE>()?;

        let position = r.read_vec3()?;

        let _ = r.read_i32::<LE>()?; // usually == -1
        let _ = r.read_i32::<LE>()?; // usually == 0.0

        let tex_coord = r.read_vec2()?;

        let normal = r.read_vec3()?;

        Ok(Vertex {
            index,
            position,
            tex_coord,
            normal,
        })
    }
}

#[derive(Debug)]
pub struct BoundingBox {
    pub max: Vec3,
    pub min: Vec3,
    pub u0: f32,
}

impl BoundingBox {
    fn read(c: &mut impl Reader) -> std::io::Result<Self> {
        let max = c.read_vec3()?;
        let min = c.read_vec3()?;
        let u0 = c.read_f32::<LE>()?;

        Ok(BoundingBox { max, min, u0 })
    }
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub parent_name: String,
    pub bone_index: u32,
    pub position: Vec3,
    pub rotation: Quat,
    pub meshes: Vec<Mesh>,
    pub bounding_boxes: Vec<BoundingBox>,
}

impl Node {
    fn read(r: &mut impl Reader, smf_version: u32) -> std::io::Result<Self> {
        let name = r.read_fixed_string(128)?;
        let parent_name = r.read_fixed_string(128)?;

        let bone_index = r.read_u32::<LE>()?; // usually == 0.0

        let position = r.read_vec3()?;
        let rotation = r.read_quat()?;

        let mesh_count = r.read_u32::<LE>()?;
        let bounding_box_count = r.read_u32::<LE>()?;

        if smf_version > 1 {
            let _ = r.read_u32::<LE>()?;
        }

        let mut meshes = Vec::with_capacity(mesh_count as usize);
        for _ in 0..mesh_count {
            meshes.push(Mesh::read(r)?);
        }

        let mut bounding_boxes = Vec::with_capacity(bounding_box_count as usize);
        for _ in 0..bounding_box_count {
            bounding_boxes.push(BoundingBox::read(r)?);
        }

        Ok(Node {
            name,
            parent_name,
            bone_index,
            position,
            rotation,
            meshes,
            bounding_boxes,
        })
    }
}

fn smf_version(s: &str) -> Option<u32> {
    Some(match s {
        s if s.starts_with("SMF V1.0") => 1,
        s if s.starts_with("SMF V1.1") => 2,
        _ => return None,
    })
}
