use clap::Parser;
use shadow_company_tools::config::{Config, ConfigReader};
use shadow_company_tools_derive::Config;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Opts {
    /// Path to the "<Shadow Company>\Data" directory.
    data_dir: PathBuf,
}

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
    #[config("BASENAME")]
    pub base_name: String,
    #[config("TITLE")]
    pub title: String,
    #[config("MULTIPLAYER_ACTIVE")]
    pub multiplayer_active: bool,
    #[config("EXCLUDE_FROM_CAMPAIGN_TREE")]
    pub exclude_from_campaign_tree: bool,
    #[config("SKIP_TEAM_EQUIPMENT_VALIDATION")]
    pub skip_team_equipment_validation: bool,
    #[config("DISABLE_HELP_TIPS")]
    disable_help_tips: bool,
    #[config("PLAYTEST_FUNDS")]
    pub playtest_funds: u32,
    #[config("MULTIPLAYER_FUNDS")]
    pub multiplayer_funds: [u32; 3],
    #[config("CUTSCENE")]
    pub cutscene: String,
    #[config("DISABLE_TEAM_AND_EQUIPPING")]
    pub disable_team_and_equipping: String,
    #[config("LIGHTING_THRESHHOLDS")]
    pub lighting_threshholds: [u32; 2],
    #[config("ENEMY_GRENADE_USE_CHANCE")]
    pub enemy_grenade_use_chance: u32,
    #[config("ALARM_AUDIO")]
    pub alarm_audio: String,
    #[config("EMITTER_CONFIG")]
    pub emitter_configs: Vec<EmitterConfig>,
    #[config("CLOTHING_INFILTRATION_MOD")]
    pub clothing_infiltration_mods: Vec<ClothingInfiltrationMod>,
    #[config("PRE_ACTION")]
    pub pre_actions: Vec<Action>,
    #[config("POST_ACTION")]
    pub post_actions: Vec<Action>,
    #[config("PRECONDITION")]
    pub preconditions: Vec<Action>,
}

#[derive(Config, Debug, Default)]
pub struct Campaigns {
    #[config("CAMPAIGN_DEF", start)]
    campaign_defs: Vec<CampaignDef>,
}

fn main() {
    let opts = Opts::parse();

    let fm = shadow_company_tools::fm::FileManager::new(opts.data_dir);

    let file = match fm.open_file("config\\campaign_defs.txt") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    };

    let mut reader = ConfigReader::new(file).expect("failed to create config reader.");
    let campaigns = match Campaigns::from_config(&mut reader) {
        Ok(campaigns) => campaigns,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    use prettytable::{row, Table};

    let mut table = Table::new();
    table.add_row(row![
        "Title",
        "Basename",
        "Multiplayer",
        "In campaign",
        "Cutscene",
        "Grenade use chance",
    ]);
    for campaign in campaigns.campaign_defs {
        table.add_row(row!(
            campaign.title,
            campaign.base_name,
            campaign.multiplayer_active,
            !campaign.exclude_from_campaign_tree,
            campaign.cutscene,
            campaign.enemy_grenade_use_chance,
        ));
    }
    table.printstd();
}
