#[macro_use] extern crate log;

use pathfinder_view::{*};
use pathfinder_renderer::scene::Scene;
use pathfinder_geometry::transform2d::Transform2F;
use svg_dom::Svg;
use svg_draw::DrawSvg;

pub struct SvgView {
    svg: DrawSvg,
}
impl Interactive for SvgView {
    type Event = Vec<u8>;
    fn title(&self) -> String {
        "SVG".into()
    }
    fn num_pages(&self) -> usize {
        1
    }
    fn scene(&mut self, page_nr: usize) -> Scene {
        self.svg.compose_with_transform(Transform2F::from_scale(25.4 / 75.))
    }
    fn event(&mut self, ctx: &mut Context, event: Vec<u8>) {
        match Svg::from_data(&event) {
            Ok(svg) => self.svg = DrawSvg::new(svg),
            Err(e) => {}
        }
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::Uint8Array;

#[cfg(target_arch = "wasm32")]
use web_sys::{HtmlCanvasElement};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info);
    warn!("test");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn show(canvas: HtmlCanvasElement, data: &Uint8Array) -> WasmView {
    use pathfinder_resources::embedded::EmbeddedResourceLoader;

    let data: Vec<u8> = data.to_vec();
    let view = SvgView { svg: DrawSvg::new(Svg::from_data(&data).unwrap()) };

    let mut config = Config::new(Box::new(EmbeddedResourceLoader));
    config.zoom = false;
    config.pan = false;
    WasmView::new(
        canvas,
        config,
        Box::new(view) as _
    )
}
