#![allow(dead_code)]

use clap::Parser;
use shadow_company_tools::config::{read_config_line, ConfigLine};
use shadow_company_tools_derive::Config;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Opts {
    /// Path to the campaign_defs.txt configuration file.
    config_file: PathBuf,
}

#[allow(dead_code)]
#[derive(Debug)]
struct EmitterConfig {
    pub name: String,
    pub config: String,
}

impl From<&ConfigLine> for EmitterConfig {
    fn from(value: &ConfigLine) -> Self {
        Self {
            name: value.params[0].clone(),
            config: value.params[1].clone(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct ClothingInfiltrationMod {
    pub name: String,
    pub v1: u32,
    pub v2: u32,
}

impl From<&ConfigLine> for ClothingInfiltrationMod {
    fn from(value: &ConfigLine) -> Self {
        Self {
            name: value.params[0].clone(),
            v1: value.params[1].parse().unwrap_or(0),
            v2: value.params[2].parse().unwrap_or(0),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Action {
    pub name: String,
    pub params: Vec<String>,
}

impl From<&ConfigLine> for Action {
    fn from(value: &ConfigLine) -> Self {
        Self {
            name: value.params[0].clone(),
            params: value.params[1..].to_vec(),
        }
    }
}

#[derive(Debug, Default, Config)]
struct Campaign {
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
    #[config("PRECONDITIONS")]
    pub preconditions: Vec<Action>,
}

fn main() {
    let fm = shadow_company_tools::fm::FileManager::new("C:\\Games\\shadow_company\\Data");

    let mut file = match fm.open_file("config\\campaign_defs.txt") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    };

    let mut campaigns = vec![];

    loop {
        let Some(line) = read_config_line(&mut file).unwrap() else {
            break;
        };

        if line.name == "CAMPAIGN_DEF" {
            campaigns.push(Campaign::default());
        } else if let Some(campaign) = campaigns.last_mut() {
            campaign.parse_config_line(&line);
        }
    }

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
    for campaign in campaigns {
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
