use std::time::{Instant, Duration};

use glium::{glutin::{event_loop::{EventLoop, ControlFlow}, event::Event, self, platform::run_return::EventLoopExtRunReturn}, Frame, Display};

pub struct UpdateInfo {
    pub time_since_previous: Duration,
    pub frames_since_previous: u32
}

pub struct DrawInfo{
    pub time_since_previous: Duration
}

pub fn start<Model, View>(
    mut event_loop: EventLoop<()>,
    display: Display, 
    mut model: Model,
    mut view: View,
    update: fn(&mut Model, &mut View, UpdateInfo), 
    draw: fn(&mut Frame, &Model, &mut View, DrawInfo),
    event: fn(Event<()>, &mut Model, &mut View) -> Option<ControlFlow>
) -> !
{
    let update_period = Duration::from_millis(20);
    let frame_period = Duration::from_millis(1000/120);
    let mut last_update = Instant::now();
    let mut last_frame = Instant::now();
    let mut frames_since_update: u32 = 0;

    event_loop.run_return(|ev, _, control_flow| {
        let now = Instant::now();

        let elapsed_since_update = now - last_update;
        let elapsed_since_frame = now - last_frame;
        last_frame = now;

        //update instances
        if update_period < elapsed_since_update{
            last_update = now;

            update(&mut model, &mut view, UpdateInfo{
                time_since_previous: elapsed_since_update,
                frames_since_previous: frames_since_update,
            });

            frames_since_update = 0;
        }

        let mut frame = display.draw();

        draw(&mut frame, &model, &mut view, DrawInfo{
            time_since_previous: elapsed_since_frame
        });

        frame.finish().unwrap();

        frames_since_update += 1;

        let next_frame_time = now + frame_period;

        *control_flow = event(ev, &mut model, &mut view).unwrap_or(
            glutin::event_loop::ControlFlow::WaitUntil(next_frame_time)
        );
    });

    std::process::exit(0)
}