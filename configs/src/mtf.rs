use shadow_company_tools_derive::Config;

#[derive(Config, Default)]
pub struct TimeOfDay {
    #[param(0)]
    pub hour: u32,
    #[param(1)]
    pub min: u32,
}

#[derive(Config, Default)]
pub struct Fog {
    #[param(0)]
    pub f0: f32,
    #[param(1)]
    pub f1: f32,
    #[param(2)]
    pub f2: f32,
    #[param(3)]
    pub f3: f32,
    #[param(4)]
    pub f4: f32,
    #[param(5)]
    pub f5: f32,
}

#[derive(Config, Default)]
pub struct Object {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub id: [i32; 2],
}

#[derive(Config, Default)]
pub struct Mtf {
    #[field("GAME_STATE_TIME_OF_DAY")]
    pub time_of_day: TimeOfDay,

    #[field("GAME_CONFIG_FOG_ENABLED")]
    pub fog: Fog,

    #[field("OBJECT_INVENTORY")]
    pub inventory_objects: Vec<Object>,

    #[field("OBJECT")]
    pub objects: Vec<Object>,
}
