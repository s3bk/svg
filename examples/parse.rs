use pathfinder_view::{show, Config};

use pathfinder_svg::{Svg};

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let data = std::fs::read(input).unwrap();
    let mut config = Config::default();
    config.zoom = true;
    config.pan = false;
    let svg = Svg::from_data(&data).unwrap();
    let scene = svg.compose();
    show(scene, config)
}
