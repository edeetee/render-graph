// mod renderer;

mod renderer;

const TRI_STRIP_VERTICES_RECT: &[Vertex] = &[
    Vertex {
        pos: [-1.0, 1.0]
    },
    Vertex{
        pos: [-1.0,-1.0]
    },
    Vertex{
        pos: [1.0, 1.0]
    },
    Vertex{
        pos: [1.0, -1.0]
    }
];

fn model(app: &App) -> ViewModel {
    let window_id = new_window(app, view);
    let window = app.window(window_id).unwrap();

    renderer::ViewModel::new(&window, TRI_STRIP_VERTICES_RECT)
}

fn view(_app: &App, model: &renderer::ViewModel, frame: Frame) {
    model.render_to(&mut frame.command_encoder(), frame.texture_view())
}