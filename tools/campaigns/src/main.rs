use clap::Parser;
use shadow_company_tools::config::{Config, ConfigReader};
use shadow_company_tools_configs::Campaigns;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Opts {
    /// Path to the "<Shadow Company>\Data" directory.
    data_dir: PathBuf,
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
