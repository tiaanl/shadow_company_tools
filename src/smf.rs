use crate::{
    common::{read_fixed_string, skip_sinister_header},
    io::GlamReadExt,
};
use byteorder::{LittleEndian, ReadBytesExt};
use glam::{Quat, Vec2, Vec3};

fn smf_version(s: &str) -> u32 {
    if s.starts_with("SMF V1.0") {
        return 1;
    }
    if s.starts_with("SMF V1.1") {
        return 2;
    }
    0
}

#[derive(Clone, Debug)]
pub struct Scene {
    pub name: String,
    pub scale: Vec3,
    pub nodes: Vec<Node>,
}

impl Scene {
    pub fn read<R>(r: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read + std::io::Seek,
    {
        skip_sinister_header(r)?;

        let version_string = read_fixed_string(r, 16);
        let smf_version = smf_version(&version_string);

        let name = read_fixed_string(r, 128);

        let mut scale = Vec3::ZERO;
        scale.x = r.read_f32::<LittleEndian>()?;
        scale.y = r.read_f32::<LittleEndian>()?;
        scale.z = r.read_f32::<LittleEndian>()?;

        let _ = r.read_f32::<LittleEndian>()?; // usually == 1.0
        let _ = r.read_u32::<LittleEndian>()?; // usually == 1

        let node_count = r.read_u32::<LittleEndian>()?;

        let mut nodes = Vec::with_capacity(node_count as usize);
        for _ in 0..node_count {
            nodes.push(Node::read(r, smf_version)?);
        }

        Ok(Scene { name, scale, nodes })
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub name: String,
    pub texture_name: String,
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
}

#[derive(Clone, Debug)]
pub struct Face {
    pub index: u32,
    pub indices: [u32; 3],
}

impl Face {
    fn read(c: &mut impl std::io::Read) -> Self {
        let index = c.read_u32::<LittleEndian>().unwrap();
        let i0 = c.read_u32::<LittleEndian>().unwrap();
        let i1 = c.read_u32::<LittleEndian>().unwrap();
        let i2 = c.read_u32::<LittleEndian>().unwrap();

        Face {
            index,
            indices: [i0, i1, i2],
        }
    }
}

impl Mesh {
    fn read(c: &mut impl std::io::Read) -> std::io::Result<Self> {
        let name = read_fixed_string(c, 128);
        let texture_name = read_fixed_string(c, 128);

        let vertex_count = c.read_u32::<LittleEndian>().unwrap();
        let face_count = c.read_u32::<LittleEndian>().unwrap();

        let mut vertices = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            vertices.push(Vertex::read(c)?);
        }

        let mut faces = Vec::with_capacity(face_count as usize);
        for _ in 0..face_count {
            faces.push(Face::read(c));
        }

        Ok(Mesh {
            name,
            texture_name,
            vertices,
            faces,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub index: u32,
    pub position: Vec3,
    pub tex_coord: Vec2,
    pub normal: Vec3,
}

impl Vertex {
    fn read(c: &mut impl std::io::Read) -> std::io::Result<Self> {
        let index = c.read_u32::<LittleEndian>()?;

        let position = Vec3::read(c)?;

        let _ = c.read_i32::<LittleEndian>()?; // usually == -1
        let _ = c.read_i32::<LittleEndian>()?; // usually == 0.0

        let mut tex_coord = Vec2::ZERO;
        tex_coord.x = c.read_f32::<LittleEndian>()?;
        tex_coord.y = c.read_f32::<LittleEndian>()?;

        let normal = Vec3::read(c)?;

        Ok(Vertex {
            index,
            position,
            tex_coord,
            normal,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CollisionBox {
    pub max: Vec3,
    pub min: Vec3,
    pub u0: f32,
}

impl CollisionBox {
    fn read(c: &mut impl std::io::Read) -> Self {
        let mut max = Vec3::ZERO;
        max.x = c.read_f32::<LittleEndian>().unwrap();
        max.y = c.read_f32::<LittleEndian>().unwrap();
        max.z = c.read_f32::<LittleEndian>().unwrap();
        let mut min = Vec3::ZERO;
        min.x = c.read_f32::<LittleEndian>().unwrap();
        min.y = c.read_f32::<LittleEndian>().unwrap();
        min.z = c.read_f32::<LittleEndian>().unwrap();
        let u0 = c.read_f32::<LittleEndian>().unwrap();

        CollisionBox { max, min, u0 }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub parent_name: String,
    pub bone_index: u32,
    pub position: Vec3,
    pub rotation: Quat,
    pub meshes: Vec<Mesh>,
    pub collision_boxes: Vec<CollisionBox>,
}

impl Node {
    fn read(c: &mut impl std::io::Read, smf_version: u32) -> std::io::Result<Self> {
        let name = read_fixed_string(c, 128);
        let parent_name = read_fixed_string(c, 128);

        let bone_index = c.read_u32::<LittleEndian>()?; // usually == 0.0

        let position = Vec3::read(c)?;
        let rotation = Quat::read(c)?;

        let mesh_count = c.read_u32::<LittleEndian>()?;
        let collision_box_count = c.read_u32::<LittleEndian>()?;

        if smf_version > 1 {
            let _ = c.read_u32::<LittleEndian>()?;
        }

        let mut meshes = Vec::with_capacity(mesh_count as usize);
        for _ in 0..mesh_count {
            meshes.push(Mesh::read(c)?);
        }

        let mut collision_boxes = Vec::with_capacity(collision_box_count as usize);
        for _ in 0..collision_box_count {
            collision_boxes.push(CollisionBox::read(c));
        }

        Ok(Node {
            name,
            parent_name,
            bone_index,
            position,
            rotation,
            meshes,
            collision_boxes,
        })
    }
}
