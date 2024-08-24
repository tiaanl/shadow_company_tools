use shadow_company_tools_derive::Config;

#[derive(Config, Debug, Default)]
pub struct UserIVar {
    #[param(0)]
    pub name: String,
    #[param(1)]
    pub value: i32,
}

#[derive(Config, Debug, Default)]
pub struct ButtonAdvice {
    #[param(0)]
    pub name: String,
    #[param(1)]
    pub x: i32,
    #[param(2)]
    pub y: i32,
    #[param(3)]
    pub dx: i32,
    #[param(4)]
    pub dy: i32,
}

#[derive(Config, Debug, Default)]
pub struct Vertex {
    #[param(0)]
    pub x_pos: f32,
    #[param(1)]
    pub y_pos: f32,
    #[param(2)]
    pub r: f32,
    #[param(3)]
    pub g: f32,
    #[param(4)]
    pub b: f32,
    #[param(5)]
    pub a: f32,
    #[param(6)]
    pub tu: f32,
    #[param(7)]
    pub tv: f32,
}

#[derive(Config, Debug, Default)]
pub struct Vertices {
    #[param(0)]
    pub count: i32,

    #[field("GEOMETRY_VERTEX")]
    pub vertices: Vec<Vertex>,
}

#[derive(Config, Debug, Default)]
pub struct Polygon {
    #[param(0)]
    pub i0: i32,
    #[param(1)]
    pub i1: i32,
    #[param(2)]
    pub i2: i32,
}

#[derive(Config, Debug, Default)]
pub struct Polygons {
    #[param(0)]
    pub count: i32,

    #[field("GEOMETRY_POLYGON")]
    pub polygons: Vec<Polygon>,
}

#[derive(Config, Debug, Default)]
pub struct WindowIVar {}

#[derive(Config, Debug, Default)]
pub struct Geometry {
    #[field("GEOMETRY_TEXTURE")]
    pub texture: String,
    #[field("GEOMETRY_TEXTURE_PACK_DX")]
    pub texture_pack_dx: i32,
    #[field("GEOMETRY_TEXTURE_PACK_DY")]
    pub texture_pack_dy: i32,
    #[field("GEOMETRY_BILINEAR_FILTERING")]
    pub bilinear_filtering: String,
    #[field("GEOMETRY_BLEND_MODE")]
    pub geometry_blend_mode: String,

    #[field("GEOMETRY_VERTICES")]
    pub vertices: Vertices,
    #[field("GEOMETRY_POLYGONS")]
    pub polygons: Polygons,
}

#[derive(Config, Debug, Default)]
pub struct GeometryTiled {
    #[field("GEOMETRY_JPG_NAME")]
    pub jpg_name: String,
    #[field("GEOMETRY_JPG_DIMENSIONS")]
    pub jpg_dimensions: [i32; 2],
    #[field("GEOMETRY_CHUNK_DIMENSIONS")]
    pub chunk_dimensions: [i32; 2],
}

#[derive(Config, Debug, Default)]
pub struct WindowBase {
    #[param(0)]
    pub name: String,
    #[field("WINDOW_BASE_DX")]
    pub dx: i32,
    #[field("WINDOW_BASE_DY")]
    pub dy: i32,
    #[field("WINDOW_BASE_RENDER_DX")]
    pub render_dx: i32,
    #[field("WINDOW_BASE_RENDER_DY")]
    pub render_dy: i32,
    #[field("WINDOW_BASE_RELOAD_ON_MODE_SWITCH")]
    pub reload_on_mode_switch: bool,

    #[field("DEFINE_USER_IVAR")]
    pub user_ivars: Vec<UserIVar>,

    #[field("MODIFY_USER_IVAR")]
    pub user_ivars_modify: Vec<UserIVar>,

    #[field("DEFINE_BUTTON_ADVICE")]
    pub button_advices: Vec<ButtonAdvice>,

    #[field("WINDOW_BASE_GEOMETRY", start)]
    pub geometry: Vec<Geometry>,

    #[field("WINDOW_BASE_GEOMETRY_TILED", start)]
    pub geometry_tiled: Vec<GeometryTiled>,
}

#[derive(Config, Debug, Default)]
pub struct WindowBases {
    #[field("WINDOW_BASE")]
    pub window_bases: Vec<WindowBase>,
}

#[cfg(test)]
mod tests {
    use shadow_company_tools::config::{Config, ConfigReader};

    use super::*;

    #[test]
    fn parse() {
        let data = r#"
            ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
            ; main_menu.txt
            ;
            ; This configuration file defines the main menu window
            ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

            WINDOW_BASE main_menu
            WINDOW_BASE_DX          640
            WINDOW_BASE_DY          480
            WINDOW_BASE_RENDER_DX   0
            WINDOW_BASE_RENDER_DY   0

            DEFINE_BUTTON_ADVICE    view_container  0       0       640     480

            DEFINE_USER_IVAR button_offset_x    30
            DEFINE_USER_IVAR button_offset_y    5

            DEFINE_BUTTON_ADVICE    b_new_game      325     80      0       0
            DEFINE_BUTTON_ADVICE    b_load_game     320     120     0       0

            WINDOW_BASE_GEOMETRY_TILED
                GEOMETRY_JPG_NAME               frame2
                GEOMETRY_JPG_DIMENSIONS         640     512
                GEOMETRY_CHUNK_DIMENSIONS       128     128

            WINDOW_BASE_GEOMETRY_TILED
                GEOMETRY_JPG_NAME               frame1
                GEOMETRY_JPG_DIMENSIONS         640     512
                GEOMETRY_CHUNK_DIMENSIONS       128     128

            WINDOW_BASE_GEOMETRY
                GEOMETRY_TEXTURE                    interface_commando_1_ck.bmp
                GEOMETRY_TEXTURE_PACK_DX            128
                GEOMETRY_TEXTURE_PACK_DY            128
                GEOMETRY_BILINEAR_FILTERING         off

                ; helper line               x       y       r       g       b       a       u       v
                GEOMETRY_VERTICES           4
                    GEOMETRY_VERTEX         0       34      1.0     1.0     1.0     1.0     0       34
                    GEOMETRY_VERTEX         128     34      1.0     1.0     1.0     1.0     128     34
                    GEOMETRY_VERTEX         128     50      1.0     1.0     1.0     1.0     128     50
                    GEOMETRY_VERTEX         0       50      1.0     1.0     1.0     1.0     0       50

                GEOMETRY_POLYGONS           2
                    GEOMETRY_POLYGON        0       1       2
                    GEOMETRY_POLYGON        2       3       0
        "#;

        let cursor = std::io::Cursor::new(data);
        let mut reader = ConfigReader::new(cursor).expect("Failed to create config reader.");

        let window_bases =
            WindowBases::from_config(&mut reader).expect("Failed to read window base config file.");
        assert_eq!(window_bases.window_bases.len(), 1);
        let window_base = window_bases.window_bases.first().unwrap();
        assert_eq!(window_base.name.as_str(), "main_menu");
        assert_eq!(window_base.dx, 640);
        assert_eq!(window_base.dy, 480);
        assert_eq!(window_base.render_dx, 0);
        assert_eq!(window_base.render_dy, 0);

        assert_eq!(window_base.user_ivars.len(), 2);
        assert_eq!(window_base.user_ivars[0].name.as_str(), "button_offset_x");
        assert_eq!(window_base.user_ivars[0].value, 30);
        assert_eq!(window_base.user_ivars[1].name.as_str(), "button_offset_y");
        assert_eq!(window_base.user_ivars[1].value, 5);

        // pub user_ivars_modify: Vec<UserIVar>,
        // pub button_advices: Vec<ButtonAdvice>,
        // pub geometry: Vec<Geometry>,

        assert_eq!(window_base.geometry_tiled.len(), 2);
        assert_eq!(window_base.geometry_tiled[0].jpg_name.as_str(), "frame2");
        assert_eq!(window_base.geometry_tiled[0].jpg_dimensions, [640, 512]);
        assert_eq!(window_base.geometry_tiled[0].chunk_dimensions, [128, 128]);
        assert_eq!(window_base.geometry_tiled[1].jpg_name.as_str(), "frame1");
        assert_eq!(window_base.geometry_tiled[1].jpg_dimensions, [640, 512]);
        assert_eq!(window_base.geometry_tiled[1].chunk_dimensions, [128, 128]);
    }
}
