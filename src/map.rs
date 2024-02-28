use crate::config::{read_config_line, ConfigLine};

pub struct Object {
    pub group_name: String,
    pub model_name: String,
    pub title: String,
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32),
    pub id: (i32, i32),
}

impl Object {
    pub fn new(group_name: String, model_name: String, title: String) -> Self {
        Self {
            group_name,
            model_name,
            title,
            position: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            id: (0, 0),
        }
    }
}

#[derive(Default)]
pub struct Map {
    pub time_of_day: (u32, u32),
    pub fog: ((f32, f32, f32), f32, f32),
    pub objects: Vec<Object>,
}

fn object_line(object: &mut Object, line: &ConfigLine) {
    if line.name == "OBJECT_POSITION" {
        let x: f32 = line.params[0].parse().unwrap_or(0.0);
        let y: f32 = line.params[1].parse().unwrap_or(0.0);
        let z: f32 = line.params[2].parse().unwrap_or(0.0);

        object.position = (x, y, z);
    } else if line.name == "OBJECT_ROTATION" {
        let x: f32 = line.params[0].parse().unwrap_or(0.0);
        let y: f32 = line.params[1].parse().unwrap_or(0.0);
        let z: f32 = line.params[2].parse().unwrap_or(0.0);

        object.rotation = (x, y, z);
    } else if line.name == "OBJECT_ID" {
        object.id = (
            line.params[0].parse().unwrap(),
            line.params[1].parse().unwrap(),
        );
    } else {
        unreachable!(
            "invalid object key: {}({})",
            line.name,
            line.params.join(", ")
        );
    }
}

impl Map {
    pub fn load<R>(&mut self, mtf_file: &mut R) -> std::io::Result<()>
    where
        R: std::io::Read + std::io::Seek,
    {
        while let Some(line) = read_config_line(mtf_file)? {
            if line.name == "OBJECT" || line.name == "OBJECT_INVENTORY" {
                self.objects.push(Object::new(
                    line.params[0].clone(),
                    line.params[1].clone(),
                    line.params[2].clone(),
                ));
            } else if line.name == "GAME_STATE_TIME_OF_DAY" {
                self.time_of_day = (
                    line.params[0].parse().unwrap_or(0),
                    line.params[0].parse().unwrap_or(0),
                );
            } else if line.name == "GAME_CONFIG_FOG_ENABLED" {
                self.fog = (
                    (
                        line.params[0].parse().unwrap_or(0.0),
                        line.params[1].parse().unwrap_or(0.0),
                        line.params[2].parse().unwrap_or(0.0),
                    ),
                    line.params[3].parse().unwrap_or(0.0),
                    line.params[4].parse().unwrap_or(0.0),
                )
            } else {
                if let Some(object) = self.objects.last_mut() {
                    object_line(object, &line);
                }
            }
        }

        Ok(())
    }
}
