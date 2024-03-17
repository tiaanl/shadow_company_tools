use crate::common::{self, read_fixed_string};
use byteorder::{LittleEndian, ReadBytesExt};

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
    pub scale: (f32, f32, f32),
    pub nodes: Vec<Node>,
}

impl Scene {
    pub fn read<R>(r: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read + std::io::Seek,
    {
        common::skip_sinister_header(r)?;

        let version_string = read_fixed_string(r, 16);
        let smf_version = smf_version(&version_string);

        let name = read_fixed_string(r, 128);

        let scale_x = r.read_f32::<LittleEndian>()?;
        let scale_y = r.read_f32::<LittleEndian>()?;
        let scale_z = r.read_f32::<LittleEndian>()?;

        let _ = r.read_f32::<LittleEndian>()?; // usually == 1.0
        let _ = r.read_u32::<LittleEndian>()?; // usually == 1

        let node_count = r.read_u32::<LittleEndian>()?;

        let mut nodes = Vec::with_capacity(node_count as usize);
        for _ in 0..node_count {
            nodes.push(Node::read(r, smf_version));
        }

        Ok(Scene {
            name,
            scale: (scale_x, scale_y, scale_z),
            nodes,
        })
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
    fn read(c: &mut impl std::io::Read) -> Self {
        let name = read_fixed_string(c, 128);
        let texture_name = read_fixed_string(c, 128);

        let vertex_count = c.read_u32::<LittleEndian>().unwrap();
        let face_count = c.read_u32::<LittleEndian>().unwrap();

        let mut vertices = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            vertices.push(Vertex::read(c));
        }

        let mut faces = Vec::with_capacity(face_count as usize);
        for _ in 0..face_count {
            faces.push(Face::read(c));
        }

        Mesh {
            name,
            texture_name,
            vertices,
            faces,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub index: u32,
    pub position: (f32, f32, f32),
    pub tex_coord: (f32, f32),
    pub normal: (f32, f32, f32),
}

impl Vertex {
    fn read(c: &mut impl std::io::Read) -> Self {
        let index = c.read_u32::<LittleEndian>().unwrap();
        let x = c.read_f32::<LittleEndian>().unwrap();
        let y = c.read_f32::<LittleEndian>().unwrap();
        let z = c.read_f32::<LittleEndian>().unwrap();
        let _ = c.read_i32::<LittleEndian>().unwrap(); // usually == -1
        let _ = c.read_i32::<LittleEndian>().unwrap(); // usually == 0.0
        let u = c.read_f32::<LittleEndian>().unwrap();
        let v = c.read_f32::<LittleEndian>().unwrap();
        let n_x = c.read_f32::<LittleEndian>().unwrap();
        let n_y = c.read_f32::<LittleEndian>().unwrap();
        let n_z = c.read_f32::<LittleEndian>().unwrap();

        Vertex {
            index,
            position: (x, y, z),
            tex_coord: (u, v),
            normal: (n_x, n_y, n_z),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CollisionBox {
    pub u1: f32,
    pub u2: f32,
    pub u3: f32,
    pub u4: f32,
    pub u5: f32,
    pub u6: f32,
    pub u7: f32,
}

impl CollisionBox {
    fn read(c: &mut impl std::io::Read) -> Self {
        let u1 = c.read_f32::<LittleEndian>().unwrap();
        let u2 = c.read_f32::<LittleEndian>().unwrap();
        let u3 = c.read_f32::<LittleEndian>().unwrap();
        let u4 = c.read_f32::<LittleEndian>().unwrap();
        let u5 = c.read_f32::<LittleEndian>().unwrap();
        let u6 = c.read_f32::<LittleEndian>().unwrap();
        let u7 = c.read_f32::<LittleEndian>().unwrap();

        CollisionBox {
            u1,
            u2,
            u3,
            u4,
            u5,
            u6,
            u7,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub parent_name: String,
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub meshes: Vec<Mesh>,
    pub collision_boxes: Vec<CollisionBox>,
}

impl Node {
    fn read(c: &mut impl std::io::Read, smf_version: u32) -> Self {
        let name = read_fixed_string(c, 128);
        let parent_name = read_fixed_string(c, 128);

        let _ = c.read_u32::<LittleEndian>().unwrap(); // usually == 0.0

        let px = c.read_f32::<LittleEndian>().unwrap();
        let py = c.read_f32::<LittleEndian>().unwrap();
        let pz = c.read_f32::<LittleEndian>().unwrap();
        let position = (px, py, pz);

        // Quaternion.
        let rx = c.read_f32::<LittleEndian>().unwrap();
        let ry = c.read_f32::<LittleEndian>().unwrap();
        let rz = c.read_f32::<LittleEndian>().unwrap();
        let rw = c.read_f32::<LittleEndian>().unwrap();
        let rotation = (rx, ry, rz, rw);

        let mesh_count = c.read_u32::<LittleEndian>().unwrap();
        let collision_box_count = c.read_u32::<LittleEndian>().unwrap();

        if smf_version > 1 {
            let _ = c.read_u32::<LittleEndian>().unwrap();
        }

        let mut meshes = Vec::with_capacity(mesh_count as usize);
        for _ in 0..mesh_count {
            meshes.push(Mesh::read(c));
        }

        let mut collision_boxes = Vec::with_capacity(collision_box_count as usize);
        for _ in 0..collision_box_count {
            collision_boxes.push(CollisionBox::read(c));
        }

        Node {
            name,
            parent_name,
            position,
            rotation,
            meshes,
            collision_boxes,
        }
    }
}
