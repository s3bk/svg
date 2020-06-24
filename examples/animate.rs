use pathfinder_view::{show, Config, Interactive, Context, Emitter};
use pathfinder_renderer::scene::Scene;
use pathfinder_svg::{Svg, DrawOptions, DrawContext, Time};
use std::time::Instant;

struct AnimatedSvg {
    svg: Svg,
    start: Instant
}

impl Interactive for AnimatedSvg {
    fn num_pages(&self) -> usize { 1 }
    fn init(&mut self, ctx: &mut Context, sender: Emitter<Self::Event>) {
        self.start = Instant::now();
        ctx.update_interval = Some(0.02);
    }
    fn idle(&mut self, ctx: &mut Context) {
        ctx.update_scene();
    }
    fn scene(&mut self, _: usize) -> Scene {
        let ctx = DrawContext::new(&self.svg);
        let mut options = DrawOptions::new(&ctx);
        options.time = Time::from_seconds(self.start.elapsed().as_secs_f64());
        self.svg.compose_with_options(&options)
    }
}

fn main() {
    env_logger::init();
    let input = std::env::args().nth(1).unwrap();
    let data = std::fs::read(input).unwrap();
    let mut config = Config::default();
    config.zoom = true;
    config.pan = false;
    let svg = Svg::from_data(&data).unwrap();
    show(AnimatedSvg {
        svg,
        start: Instant::now()
    }, config)
}
