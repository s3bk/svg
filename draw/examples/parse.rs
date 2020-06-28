use svg_dom::{Svg};

fn main() {
    env_logger::init();
    let input = std::env::args().nth(1).unwrap();
    let data = std::fs::read(input).unwrap();
    let svg = Svg::from_data(&data).unwrap();
}
