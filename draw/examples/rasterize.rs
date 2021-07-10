use svg_dom::{Svg};
use svg_draw::{DrawContext};
use svg_text::{FontCollection, Font};
use std::sync::Arc;
use pathfinder_rasterize::Rasterizer;
use pathfinder_color::ColorF;
use pathfinder_renderer::gpu::options::RendererLevel;

fn main() {
    env_logger::init();
    let mut args = std::env::args().skip(1);
    let input = args.next().unwrap();
    let data = std::fs::read(input).unwrap();
    let output = args.next().unwrap();

    let fonts = FontCollection::from_fonts(vec![
        Font::load(include_bytes!("../../resources/latinmodern-math.otf")),
        Font::load(include_bytes!("../../resources/NotoNaskhArabic-Regular.ttf")),
        Font::load(include_bytes!("../../resources/NotoSerifBengali-Regular.ttf")),
    ]);

    let svg = Svg::from_data(&data).unwrap();
    let scene = DrawContext::new(&svg, &fonts).compose();
    let image = Rasterizer::new_with_level(RendererLevel::D3D9).rasterize(scene, Some(ColorF::white()));
    image.save(&output).unwrap();
}
