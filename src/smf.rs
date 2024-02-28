use crate::common::read_fixed_string;
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

#[derive(Debug)]
pub struct Scene {
    pub name: String,
    pub scale: (f32, f32, f32),
    pub models: Vec<Model>,
}

impl Scene {
    pub fn read<R>(r: &mut R) -> Self
    where
        R: std::io::Read + std::io::Seek,
    {
        let version_string = read_fixed_string(r, 16);
        let smf_version = smf_version(&version_string);

        let name = read_fixed_string(r, 128);

        let scale_x = r.read_f32::<LittleEndian>().unwrap();
        let scale_y = r.read_f32::<LittleEndian>().unwrap();
        let scale_z = r.read_f32::<LittleEndian>().unwrap();

        // println!("scale: ({scale_x:.2}, {scale_y:.2}, {scale_z:.2})");

        let _u1 = r.read_u32::<LittleEndian>().unwrap();
        // println!("unknown: {:08X}", _u1);
        let _u2 = r.read_u32::<LittleEndian>().unwrap();
        // println!("unknown: {:08X}", _u2);

        let sub_model_count = r.read_u32::<LittleEndian>().unwrap();

        // SubModel

        let mut sub_models = vec![];
        for _ in 0..sub_model_count {
            sub_models.push(Model::read(r, smf_version));
        }

        Scene {
            name,
            scale: (scale_x, scale_y, scale_z),
            models: sub_models,
        }
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
        let quad_count = c.read_u32::<LittleEndian>().unwrap();

        let mut vertices = vec![];
        for _ in 0..vertex_count {
            vertices.push(Vertex::read(c));
        }

        let mut quads = vec![];
        for _ in 0..quad_count {
            quads.push(Face::read(c));
        }

        Mesh {
            name,
            texture_name,
            vertices,
            faces: quads,
        }
    }
}

#[derive(Debug)]
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
        let _ = c.read_i32::<LittleEndian>().unwrap(); // usualle == -1
        let _ = c.read_f32::<LittleEndian>().unwrap(); // usually == 0.0
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Model {
    pub name: String,
    pub parent_name: String,
    pub position: (f32, f32, f32),
    pub meshes: Vec<Mesh>,
    pub collision_boxes: Vec<CollisionBox>,
}

impl Model {
    fn read(c: &mut impl std::io::Read, smf_version: u32) -> Self {
        let name = read_fixed_string(c, 128);
        let parent_name = read_fixed_string(c, 128);

        let _u1 = c.read_f32::<LittleEndian>().unwrap(); // usuallt == 0.0

        let position_x = c.read_f32::<LittleEndian>().unwrap();
        let position_y = c.read_f32::<LittleEndian>().unwrap();
        let position_z = c.read_f32::<LittleEndian>().unwrap();

        // -0.50, 0.50, -0.50, -0.50
        // -0.50, 0.50,  0.50,  0.50
        let _u5 = c.read_f32::<LittleEndian>().unwrap();
        let _u6 = c.read_f32::<LittleEndian>().unwrap();
        let _u7 = c.read_f32::<LittleEndian>().unwrap();
        let _u8 = c.read_f32::<LittleEndian>().unwrap();

        let geometry_count = c.read_u32::<LittleEndian>().unwrap();
        let collision_box_count = c.read_u32::<LittleEndian>().unwrap();

        if smf_version > 1 {
            let _ = c.read_u32::<LittleEndian>().unwrap();
        }

        let mut geometries = vec![];
        for _ in 0..geometry_count {
            geometries.push(Mesh::read(c));
        }

        let mut sub_sub_2 = vec![];
        for _ in 0..collision_box_count {
            sub_sub_2.push(CollisionBox::read(c));
        }

        Model {
            name,
            parent_name,
            position: (position_x, position_y, position_z),
            meshes: geometries,
            collision_boxes: sub_sub_2,
        }
    }
}
