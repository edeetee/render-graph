use glium_utils::glam::Mat4;
use nannou_osc as osc;
use nannou_osc::rosc::OscType;
use osc::Receiver;
use super::stars::Stars;

pub struct Options {
    pub speed: f32
}

impl Default for Options{
    fn default() -> Self {
        Self { speed: 1. }
    }
}

pub struct Model {
    pub stars: Stars,
    pub mat: [[f32; 4]; 4],
    receiver: Receiver,
    options: Options,
    pub feedback_displace: [f32; 2]
}

const PORT: u16 = 10000;

impl Model {
    pub fn new(num_stars: usize, options: Option<Options>) -> Self {
        let stars = Stars::new(num_stars);
        let perspective = Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_2, 
            1., 
            0.001, 
            1000.
        );

        let feedback_displace: [f32; 2] = [0.0, 0.2];
     
        Self { 
            stars, 
            mat: perspective.to_cols_array_2d(), 

            feedback_displace,
    
            receiver: osc::receiver(PORT).unwrap(),
    
            options: options.unwrap_or(Options::default()),
            // perf_record: PerformanceRecord::default()
        }
    }

    pub fn update(&mut self, step_seconds: f32){
        self.stars.update(step_seconds*self.options.speed);

        for (packet, _) in self.receiver.try_iter(){
            for msg in packet.into_msgs(){
                match msg.addr.as_str() {
                    "/speed" => {
                        if let Some(speed) = msg.args.map(parse_float).flatten(){
                            self.options.speed = speed;
                            println!("osc speed: {speed:.2}");
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn parse_float<'a>(args: Vec<OscType>) -> Option<f32> {
    if let OscType::Float(result) = args.first()?{
        Some(result.clone())
    } else {
        None
    }
}