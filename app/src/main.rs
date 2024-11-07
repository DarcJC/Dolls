use spdlog::info;

fn main() {
    init_logger();
    info!("Running dolls server");
}

fn init_logger() {
    spdlog::default_logger().set_level_filter(spdlog::LevelFilter::All);
}
