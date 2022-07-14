use std::iter;
use glam::Vec3;
use palette::{rgb::Rgb, Pixel, IntoColor, LinSrgb};
use rand::{prelude::*, distributions::uniform::{SampleUniform, SampleRange}};

pub use palette::Hsv;

#[derive(Debug, Default)]
pub struct Star{
    ///position
    pub pos: Vec3,
    // pub tx: Mat4,
    
    ///velocity per second
    vel: Vec3,

    ///radius for draw
    pub radius: f32,

    //color for draw
    pub color: [f32; 3],
}

fn random_range<T,R>(range: R) -> T
where
    T: SampleUniform,
    R: SampleRange<T>, 
{
    rand::thread_rng().gen_range(range)
}

impl Star{

    fn new_rand() -> Self {
       let mut new_star = Self::default();
       new_star.reset();
       new_star
    }

    fn update(&mut self, seconds: f32) {
        let delta = self.vel*seconds;
        
        self.pos += delta;
        // self.tx.transform_point3(delta);
        // self.vel = (self.vel*10. + Self::rand_vel()).normalize();
    }

    fn reset(&mut self) {
        self.rand_pos();
        self.rand_vel();
        self.rand_radius();
        self.rand_color();
    }

    fn rand_pos(&mut self) {
        // self.tx
        self.pos.x = random_range(-POS_OFFSETXY..POS_OFFSETXY);
        self.pos.y = random_range(-POS_OFFSETXY..POS_OFFSETXY);
        self.pos.z = random_range(-100f32..-0.5f32);
    }

    fn rand_vel(&mut self) {
        self.vel.x = random_range(-VEL_OFFSET..VEL_OFFSET);
        self.vel.y = random_range(-VEL_OFFSET..VEL_OFFSET);
        self.vel.z = 1.;

        self.vel = self.vel.normalize();
    }

    fn rand_color(&mut self){
        let hsv = Hsv::new(
            random_range(-180.0..180.0),
            random_range(0.5..0.9),
            random_range(0.7..1.0)
        );

        let rgb: Rgb = hsv.into_color();
        self.color = rgb.into_raw();
        // self.color.r = rand::thread_rng().gen_range(COLOR_RANGE);
        // self.color.g = rand::thread_rng().gen_range(COLOR_RANGE);
        // self.color.b = rand::thread_rng().gen_range(COLOR_RANGE);
    }

    fn rand_radius(&mut self) { 
        self.radius = random_range(0.5f32..10f32) 
    }
}

const VEL_OFFSET: f32 = 0.05;
const POS_OFFSETXY: f32 = 100.;


pub struct Stars {
    stars: Vec<Star>
}

impl IntoIterator for Stars {
    type Item = Star;

    type IntoIter = std::vec::IntoIter<Star>;

    fn into_iter(self) -> Self::IntoIter {
        self.stars.into_iter()
    }
}

impl Stars {
    pub fn new(num_stars: usize) -> Self {
        Self {
            stars: iter::repeat_with(Star::new_rand).take(num_stars).collect()
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Star> {
        self.stars.iter()
    }

    pub fn update(&mut self, seconds: f32) {
        for star in self.stars.iter_mut(){
            star.update(seconds);

            if 0. < star.pos.z {
                star.reset();
            }
            // println!("pos: {}", star.pos);
        }
    }
}