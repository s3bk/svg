use pathfinder_view::{show, Config, Interactive, Context, Emitter};
use pathfinder_renderer::scene::Scene;
//use pathfinder_resources::embedded::EmbeddedResourceLoader;
use pathfinder_resources::fs::FilesystemResourceLoader;
use std::path::PathBuf;
use svg_dom::{Svg, Time};
use svg_draw::{DrawSvg, DrawOptions, DrawContext};
use pathfinder_geometry::{
    vector::Vector2F,
    transform2d::Transform2F,
    rect::RectF,
};
use svg_text::{FontCollection, Font};
use std::sync::Arc;

fn main() {
    env_logger::init();
    let input = std::env::args().nth(1).unwrap();
    let data = std::fs::read(input).unwrap();
    //let resource_loader = EmbeddedResourceLoader;
    let resource_loader = FilesystemResourceLoader { directory: PathBuf::from("/home/sebk/Rust/pathfinder/resources") };
    let mut config = Config::new(Box::new(resource_loader));
    config.zoom = true;
    config.pan = true;

    let fonts = Arc::new(FontCollection::from_fonts(vec![
        Font::load(include_bytes!("../../resources/latinmodern-math.otf")),
        Font::load(include_bytes!("../../resources/NotoNaskhArabic-Regular.ttf")),
        Font::load(include_bytes!("../../resources/NotoSerifBengali-Regular.ttf")),
    ]));

    let svg = Svg::from_data(&data).unwrap();
    show(View::new(svg, fonts), config)
}

struct View {
    svg: DrawSvg,
    view_box: Option<RectF>
}
impl View {
    fn new(svg: Svg, fonts: Arc<FontCollection>) -> View {
        let svg = DrawSvg::new(svg, fonts);
        let view_box = svg.view_box();
        View {
            svg, view_box
        }
    }
}
impl Interactive for View {
    fn scene(&mut self, ctx: &mut Context) -> Scene {
        let mut scene = Scene::new();
        if let Some(vb) = self.view_box {
            scene.set_view_box(vb);
        } else {
            scene.set_view_box(RectF::new(Vector2F::zero(), ctx.window_size()))
        };
        self.svg.compose_to_with_transform(&mut scene, dbg!(ctx.view_transform()));
        scene
    }
    fn window_size_hint(&self) -> Option<Vector2F> {
        self.view_box.map(|vb| vb.size())
    }
    fn init(&mut self, ctx: &mut Context, sender: Emitter<Self::Event>) {
        ctx.set_scale(1.0);
        if let Some(vb) = self.view_box {
            ctx.set_view_box(vb);
            //ctx.set_bounds(vb);
            ctx.move_to(vb.center());
        }
    }
}