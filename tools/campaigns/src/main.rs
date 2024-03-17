use clap::Parser;
use shadow_company_tools::config::{read_config_line, ConfigLine};
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

#[allow(dead_code)]
#[derive(Debug)]
struct ClothingInfiltrationMod {
    pub name: String,
    pub v1: u32,
    pub v2: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Action {
    pub name: String,
    pub params: Vec<String>,
}

#[derive(Debug, Default)]
struct Campaign {
    pub base_name: String,
    pub title: String,
    pub multiplayer_active: bool,
    pub exclude_from_campaign_tree: bool,
    pub skip_team_equipment_validation: bool,
    pub playtest_funds: u32,
    pub multiplayer_funds: [u32; 3],
    pub cutscene: bool,
    pub disable_team_and_equipping: String,
    pub lighting_threshholds: [u32; 2],
    pub enemy_grenade_use_chance: u32,
    pub alarm_audio: String,
    pub emitter_configs: Vec<EmitterConfig>,
    pub clothing_infiltration_mods: Vec<ClothingInfiltrationMod>,
    pub pre_actions: Vec<Action>,
    pub post_actions: Vec<Action>,
    pub preconditions: Vec<Action>,
}

fn read_campaign_line(campaign: &mut Campaign, line: &ConfigLine) -> std::io::Result<bool> {
    if line.name == "BASENAME" {
        campaign.base_name = line.params[0].clone();
    } else if line.name == "TITLE" {
        campaign.title = line.params[0].clone();
    } else if line.name == "MULTIPLAYER_ACTIVE" {
        campaign.multiplayer_active = true;
    } else if line.name == "EXCLUDE_FROM_CAMPAIGN_TREE" {
        campaign.exclude_from_campaign_tree = true;
    } else if line.name == "SKIP_TEAM_EQUIPMENT_VALIDATION" {
        campaign.skip_team_equipment_validation = true;
    } else if line.name == "PLAYTEST_FUNDS" {
        campaign.playtest_funds = line.params[0].parse().unwrap_or(0);
    } else if line.name == "MULTIPLAYER_FUNDS" {
        let v1 = line.params[0].parse().unwrap_or(0);
        let v2 = line.params[0].parse().unwrap_or(0);
        let v3 = line.params[0].parse().unwrap_or(0);
        campaign.multiplayer_funds = [v1, v2, v3];
    } else if line.name == "CUTSCENE" {
        campaign.cutscene = true;
    } else if line.name == "DISABLE_TEAM_AND_EQUIPPING" {
        campaign.disable_team_and_equipping = line.params[0].clone();
    } else if line.name == "LIGHTING_THRESHHOLDS" {
        let v1 = line.params[0].parse().unwrap_or(0);
        let v2 = line.params[0].parse().unwrap_or(0);
        campaign.lighting_threshholds = [v1, v2];
    } else if line.name == "ENEMY_GRENADE_USE_CHANCE" {
        campaign.enemy_grenade_use_chance = line.params[0].parse().unwrap_or(0);
    } else if line.name == "ALARM_AUDIO" {
        campaign.alarm_audio = line.params[0].clone();
    } else if line.name == "EMITTER_CONFIG" {
        campaign.emitter_configs.push(EmitterConfig {
            name: line.params[0].clone(),
            config: line.params[1].clone(),
        });
    } else if line.name == "CLOTHING_INFILTRATION_MOD" {
        campaign
            .clothing_infiltration_mods
            .push(ClothingInfiltrationMod {
                name: line.params[0].clone(),
                v1: line.params[1].parse().unwrap_or(0),
                v2: line.params[1].parse().unwrap_or(0),
            });
    } else if line.name == "PRE_ACTION" {
        let name = line.params[0].clone();
        let params = line.params[1..].to_vec();

        campaign.pre_actions.push(Action { name, params });
    } else if line.name == "POST_ACTION" {
        let name = line.params[0].clone();
        let params = line.params[1..].to_vec();

        campaign.post_actions.push(Action { name, params });
    } else if line.name == "PRECONDITION" {
        let name = line.params[0].clone();
        let params = line.params[1..].to_vec();

        campaign.preconditions.push(Action { name, params });
    } else {
        return Ok(false);
    }
    Ok(true)
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
            read_campaign_line(campaign, &line).unwrap();
        }
    }

    for c in campaigns.iter() {
        println!("Campaign: {} ({})", c.title, c.base_name);
    }
}
