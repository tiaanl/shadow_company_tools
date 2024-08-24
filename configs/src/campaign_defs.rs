use shadow_company_tools_derive::Config;

#[derive(Config, Debug, Default)]
pub struct EmitterConfig {
    #[param(0)]
    pub name: String,
    #[param(1)]
    pub config: String,
}

#[derive(Config, Debug, Default)]
pub struct ClothingInfiltrationMod {
    #[param(0)]
    pub name: String,
    #[param(1)]
    pub v1: u32,
    #[param(2)]
    pub v2: u32,
}

#[allow(dead_code)]
#[derive(Config, Debug, Default)]
pub struct Action {
    #[param(0)]
    pub name: String,
    #[param(1)]
    pub params: Vec<String>,
}

#[derive(Config, Debug, Default)]
pub struct CampaignDef {
    #[field("BASENAME")]
    pub base_name: String,
    #[field("TITLE")]
    pub title: String,
    #[field("MULTIPLAYER_ACTIVE")]
    pub multiplayer_active: bool,
    #[field("EXCLUDE_FROM_CAMPAIGN_TREE")]
    pub exclude_from_campaign_tree: bool,
    #[field("SKIP_TEAM_EQUIPMENT_VALIDATION")]
    pub skip_team_equipment_validation: bool,
    #[field("DISABLE_HELP_TIPS")]
    disable_help_tips: bool,
    #[field("PLAYTEST_FUNDS")]
    pub playtest_funds: u32,
    #[field("MULTIPLAYER_FUNDS")]
    pub multiplayer_funds: [u32; 3],
    #[field("CUTSCENE")]
    pub cutscene: String,
    #[field("DISABLE_TEAM_AND_EQUIPPING")]
    pub disable_team_and_equipping: String,
    #[field("LIGHTING_THRESHHOLDS")]
    pub lighting_threshholds: [u32; 2],
    #[field("ENEMY_GRENADE_USE_CHANCE")]
    pub enemy_grenade_use_chance: u32,
    #[field("ALARM_AUDIO")]
    pub alarm_audio: String,
    #[field("EMITTER_CONFIG")]
    pub emitter_configs: Vec<EmitterConfig>,
    #[field("CLOTHING_INFILTRATION_MOD")]
    pub clothing_infiltration_mods: Vec<ClothingInfiltrationMod>,
    #[field("PRE_ACTION")]
    pub pre_actions: Vec<Action>,
    #[field("POST_ACTION")]
    pub post_actions: Vec<Action>,
    #[field("PRECONDITION")]
    pub preconditions: Vec<Action>,
}

#[derive(Config, Debug, Default)]
pub struct CampaignDefs {
    #[field("CAMPAIGN_DEF", start)]
    pub campaign_defs: Vec<CampaignDef>,
}
