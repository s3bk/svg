#[macro_use] extern crate log;

use pathfinder_view::{*};
use pathfinder_renderer::scene::Scene;
use pathfinder_geometry::transform2d::Transform2F;
use svg_dom::Svg;
use svg_draw::DrawContext;
use svg_text::{Font, FontCollection};
use std::sync::Arc;

use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;

use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

pub struct SvgView {
    svg: Svg,
    fonts: Arc<FontCollection>,
}
impl Interactive for SvgView {
    type Event = Vec<u8>;
    fn title(&self) -> String {
        "SVG".into()
    }
    fn scene(&mut self, ctx: &mut Context) -> Scene {
        DrawContext::new(&self.svg, &self.fonts)
        .compose_with_transform(
            Transform2F::from_scale(25.4 / 75.)
        )
    }
    fn event(&mut self, ctx: &mut Context, event: Vec<u8>) {
        match Svg::from_data(&event) {
            Ok(svg) => self.svg = svg,
            Err(e) => {}
        }
    }
}

#[wasm_bindgen]
pub struct FontBuilder(FontCollection);

#[wasm_bindgen]
#[derive(Clone)]
pub struct Fonts(Arc<FontCollection>);

#[wasm_bindgen]
impl FontBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> FontBuilder {
        FontBuilder(FontCollection::new())
    }
    pub fn add(&mut self, data: &Uint8Array) {
        let data: Vec<u8> = data.to_vec();
        self.0.add_font(Font::load(&data));
    }
    pub fn build(self) -> Fonts {
        Fonts(Arc::new(self.0))
    }
}

#[wasm_bindgen(start)]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info);
    warn!("test");
}

#[wasm_bindgen]
pub fn show(canvas: HtmlCanvasElement, context: WebGl2RenderingContext, data: &Uint8Array, fonts: &Fonts) -> WasmView {
    use pathfinder_resources::embedded::EmbeddedResourceLoader;

    let data: Vec<u8> = data.to_vec();
    let view = SvgView {
        svg: Svg::from_data(&data).unwrap(),
        fonts: fonts.0.clone(),
    };

    let mut config = Config::new(Box::new(EmbeddedResourceLoader));
    config.zoom = false;
    config.pan = false;
    WasmView::new(
        canvas,
        context,
        config,
        Box::new(view) as _
    )
}
