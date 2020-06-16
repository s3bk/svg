use pathfinder_view::{show, Config};

use pathfinder_svg::{Svg};
use roxmltree::Document;
use std::time::Instant;

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let data = std::fs::read_to_string(input).unwrap();

    let t0 = Instant::now();
    let doc = Document::parse(&data).unwrap();
    
    let t1 = Instant::now();
    let svg = Svg::parse(&doc).unwrap();

    let t2 = Instant::now();
    let scene = svg.compose();
    
    let t3 = Instant::now();

    println!("XML parse: {:?}", t1 - t0);
    println!("SVG parse: {:?}", t2 - t1);
    println!("Scene building: {:?}", t3 - t2);

    let mut config = Config::default();
    config.zoom = true;
    config.pan = false;
    show(scene, config)
}
