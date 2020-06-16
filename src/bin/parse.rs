use pathfinder_view::{show, Config};

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let data = std::fs::read_to_string(input).unwrap();
    let svg = pathfinder_svg::parse(&data).unwrap();
    let scene = svg.compose();

    let mut config = Config::default();
    config.zoom = true;
    config.pan = false;
    show(scene, config)
}
