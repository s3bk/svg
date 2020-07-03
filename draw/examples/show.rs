use pathfinder_view::{show, Config, Interactive, Context, Emitter};
use pathfinder_renderer::scene::Scene;
use svg_dom::{Svg, Time};
use svg_draw::{DrawSvg, DrawOptions, DrawContext};

fn main() {
    env_logger::init();
    let input = std::env::args().nth(1).unwrap();
    let data = std::fs::read(input).unwrap();
    let mut config = Config::default();
    config.zoom = true;
    config.pan = true;
    let svg = Svg::from_data(&data).unwrap();
    show(DrawSvg::new(svg).compose(), config)
}
