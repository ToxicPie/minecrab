use minecrab::config::GameConfiguration;
use minecrab::game::map::{GameMap, MAP_HEIGHT, MAP_WIDTH};
use minecrab::game::replay::{log_event, GameEvent};
use minecrab::kernel;

use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    config_path: String,
}

fn main() {
    let args = Args::parse();

    let game_config =
        GameConfiguration::load(&args.config_path).expect("Failed to load game config");
    let kernel_config = game_config.get_kernel_config();
    let user_configs = game_config
        .get_user_configs()
        .expect("Failed to load user config");

    let map_data = game_config.read_mapdata().expect("Failed to load map data");
    log_event(GameEvent::InitMap {
        map_data: &map_data,
        map_width: MAP_WIDTH,
        map_height: MAP_HEIGHT,
    });

    let game_map = GameMap::from_map_data(&map_data).expect("Failed to build map data");

    let mut kernel = kernel::Kernel::new(kernel_config, game_map);
    kernel.setup_users(user_configs);
    kernel.run_full_game();
}
