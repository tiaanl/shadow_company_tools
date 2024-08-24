use shadow_company_tools_derive::Config;

// SPRITEFRAME <x1> <y1> <x2> <y2>
#[derive(Config, Debug, Default)]
pub struct SpriteFrame {
    #[param(0)]
    pub x1: i32,
    #[param(1)]
    pub y1: i32,
    #[param(2)]
    pub x2: i32,
    #[param(3)]
    pub y2: i32,
}

// SPRITEFRAME_XRUN <X1> <Y1> <DX> <DY> <NUM_FRAMES>
#[derive(Config, Debug, Default)]
pub struct SpriteFrameXRun {
    #[param(0)]
    pub x1: i32,
    #[param(1)]
    pub y1: i32,
    #[param(2)]
    pub x2: i32,
    #[param(3)]
    pub y2: i32,
    #[param(4)]
    pub num_frames: i32,
}

// SPRITEFRAME_DXRUN <X1> <Y1> <SEP_DX> <DY> <NUM_FRAME> <DX * NUM_FRAMES>
#[derive(Config, Debug, Default)]
pub struct SpriteFrameDxRun {
    #[param(0)]
    pub x1: i32,
    #[param(1)]
    pub y1: i32,
    #[param(2)]
    pub sep_dx: i32,
    #[param(3)]
    pub dy: i32,
    #[param(4)]
    pub num_frame: i32,
    #[param(5)]
    pub dx: i32,
}

// SPRITE3D <NAME> <TEXTURENAME> <TXTR_WIDTH> <TXTR_HEIGHT> [<ALPHA>] [<Color Key Enable>] [ <Rl> <Gl> <Bl> <Rh> <Gh> Bh> ]
#[derive(Config, Debug, Default)]
pub struct Sprite3d {
    #[param(0)]
    pub name: String,
    #[param(1)]
    pub texture_name: String,
    #[param(2)]
    pub texture_width: i32,
    #[param(3)]
    pub texture_height: i32,
    #[param(4)]
    pub alpha: i32,

    #[field(key = "SPRITEFRAME")]
    pub sprite_frames: Vec<SpriteFrame>,
}

#[derive(Config, Debug, Default)]
pub struct Image {
    #[param(0)]
    pub name: String,
    #[param(1)]
    pub filename: String,
    #[param(2)]
    pub vid_mem: i32,
}

#[derive(Config, Debug, Default)]
pub struct FrameDescriptor {
    #[param(0)]
    pub num_images: i32,
    #[param(1)]
    pub num_frames: i32,
    #[param(2)]
    pub frame_rate: i32,
}

#[derive(Config, Debug, Default)]
pub struct FrameOrder {
    #[param(0)]
    pub order: Vec<i32>,
}

#[derive(Config, Debug, Default)]
pub struct AnimSprite {
    #[param(0)]
    pub name: String,
    #[param(1)]
    pub texture_name: String,
    #[param(2)]
    pub width: i32,
    #[param(3)]
    pub height: i32,

    #[field("FRAMEDESCRIPTOR")]
    pub frame_descriptor: FrameDescriptor,
    #[field("FRAMEORDER")]
    pub frame_orders: Vec<FrameOrder>,
    #[field("SPRITEFRAME")]
    pub sprite_frames: Vec<SpriteFrame>,
    #[field("SPRITEFRAME_XRUN")]
    pub sprite_frame_xruns: Vec<SpriteFrameXRun>,
    #[field("SPRITEFRAME_DXRUN")]
    pub sprite_frame_dxruns: Vec<SpriteFrameDxRun>,
}

#[derive(Config, Debug, Default)]
pub struct ImageDefs {
    #[field("IMAGE")]
    pub images: Vec<Image>,
    #[field("SPRITE3D", end = "ENDDEF")]
    pub sprite_3ds: Vec<Sprite3d>,
    #[field("ANIMSPRITE3D", end = "ENDDEF")]
    pub anim_sprite_3ds: Vec<AnimSprite>,
    #[field("ANIMSPRITE", end = "ENDDEF")]
    pub anim_sprites: Vec<AnimSprite>,
}
