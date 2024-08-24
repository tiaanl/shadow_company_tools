use glam::Vec3;

use crate::config::{read_config_line, ConfigLine};

#[derive(Debug, Default)]
pub struct Object {
    pub group_name: String,
    pub model_name: String,
    pub title: String,
    pub position: Vec3,
    pub rotation: (f32, f32, f32),
    pub id: (i32, i32),
}

#[derive(Debug, Default)]
pub struct Map {
    pub time_of_day: (u32, u32),
    pub fog: ((f32, f32, f32), f32, f32),
    pub objects: Vec<Object>,
}

fn object_line(object: &mut Object, line: &ConfigLine) {
    if line.name == "OBJECT_POSITION" {
        object.position = Vec3::new(
            line.param(0).unwrap_or(0.0),
            line.param(1).unwrap_or(0.0),
            line.param(2).unwrap_or(0.0),
        );
    } else if line.name == "OBJECT_ROTATION" {
        let x: f32 = line.param(0).unwrap_or(0.0);
        let y: f32 = line.param(1).unwrap_or(0.0);
        let z: f32 = line.param(2).unwrap_or(0.0);

        object.rotation = (x, y, z);
    } else if line.name == "OBJECT_ID" {
        object.id = (line.param(0).unwrap(), line.param(1).unwrap());
    } else if line.name == "OBJECT_MTF_CONFIG" {
        // println!("OBJECT_MTF_CONFIG: {:?}", line.params);
    } else {
        unreachable!("invalid object key: {}", line.name,);
    }
}

impl Map {
    pub fn load<R>(&mut self, mtf_file: &mut R) -> std::io::Result<()>
    where
        R: std::io::Read + std::io::Seek,
    {
        while let Some(line) = read_config_line(mtf_file)? {
            if line.name == "OBJECT" || line.name == "OBJECT_INVENTORY" {
                self.objects.push(Object {
                    group_name: line.param(0).unwrap(),
                    model_name: line.param(1).unwrap(),
                    title: line.param(2).unwrap(),
                    ..Default::default()
                });
                // Inventory ALITSP-Medkit "Medical Kit"
            } else if line.name == "GAME_STATE_TIME_OF_DAY" {
                self.time_of_day = (line.param(0).unwrap_or(0), line.param(1).unwrap_or(0));
            } else if line.name == "GAME_CONFIG_FOG_ENABLED" {
                self.fog = (
                    (
                        line.param(0).unwrap_or(0.0),
                        line.param(1).unwrap_or(0.0),
                        line.param(2).unwrap_or(0.0),
                    ),
                    line.param(3).unwrap_or(0.0),
                    line.param(4).unwrap_or(0.0),
                )
            } else if let Some(object) = self.objects.last_mut() {
                object_line(object, &line);
            }
        }

        Ok(())
    }
}
