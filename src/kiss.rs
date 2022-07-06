use kiss3d::{window::{Window, State}, nalgebra::{Vector3, Matrix4, Perspective3}, renderer::PointRenderer};

struct KissState {
    stars: Stars,
    point_renderer: PointRenderer,
    prev_frame: Instant
}

impl State for KissState{
    fn step(&mut self, window: &mut Window) {
        let timeElapsed = self.prev_frame.elapsed().as_secs_f32();

        self.stars.update(timeElapsed);

        // for star in self.stars.iter(){
        //     window.
        // }

        self.prev_frame = Instant::now();
    }
}

fn main_kiss() {
    let mut window = Window::new("Stars");

    let screen_scale = Vector3::new(window.width() as f32, window.height() as f32, 1.);
    
    let proj_mat = Perspective3::new((window.width() as f32)/(window.width() as f32), FRAC_PI_2, 0.01, 1000.);

    let point_renderer = PointRenderer::new();

    let stars = Stars::new();

    let state = KissState {
        stars,
        point_renderer,
        prev_frame: Instant::now()
    };

    window.render_loop(state)
}