use pathfinder_view::{show, Config};

use pathfinder_svg::{Svg};
use roxmltree::Document;

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let data = std::fs::read_to_string(input).unwrap();

    let doc = Document::parse(&data).unwrap();
    let svg = Svg::parse(&doc).unwrap();

    let scene = svg.compose();

    let mut config = Config::default();
    config.zoom = true;
    config.pan = false;
    show(scene, config)
}
