use std::{iter, time::Instant, ops::Mul};
use glam::{Vec3, Vec2};
use palette::{rgb::{Rgba}, Pixel, IntoColor, Hsva, blend::PreAlpha};
use rand::{prelude::*, distributions::uniform::{SampleUniform, SampleRange}};

pub use palette::Hsv;

use tracing::{span, Level, instrument};

#[derive(Debug, Default)]
pub struct Star{
    ///position
    pub pos: Vec3,
    // pub tx: Mat4,
    
    ///velocity per second
    vel: Vec3,

    pub scale: Vec2,

    ///radius for draw
    // pub radius: f32,

    //color for draw
    pub rgba: [f32; 4],
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
        // self.rand_radius();
        self.rand_color();
        self.rand_scale();
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

    fn rand_scale(&mut self) {
        self.scale.x = random_range(0.0..0.2);
        self.scale.y = random_range(0.0..0.2);
    }

    fn rand_color(&mut self){
        let hsv = Hsva::new(
            random_range(-180.0..180.0),
            random_range(0.0..0.8),
            1.0,
            random_range(0.0..1.0)
        );

        let rgba: Rgba = hsv.into_color();
        self.rgba = PreAlpha::from(rgba).into_raw();
        // self.color.r = rand::thread_rng().gen_range(COLOR_RANGE);
        // self.color.g = rand::thread_rng().gen_range(COLOR_RANGE);
        // self.color.b = rand::thread_rng().gen_range(COLOR_RANGE);
    }

    // fn rand_radius(&mut self) { 
    //     self.radius = random_range(0.001f32..0.1f32)
    // }
}

const VEL_OFFSET: f32 = 0.05;
const POS_OFFSETXY: f32 = 20.;

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
        let mut new_self = Self {
            stars: iter::repeat_with(Star::new_rand).take(num_stars).collect()
        };
        
        new_self.sort();

        new_self
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Star> {
        self.stars.iter()
    }

    #[instrument(skip_all)]
    fn sort(&mut self){
        self.stars.sort_unstable_by(|a, b| a.pos.z.partial_cmp(&b.pos.z).unwrap() );
    }

    pub fn update(&mut self, seconds: f32) {
        let mut changed = false;

        for star in self.stars.iter_mut(){
            star.update(seconds);

            if 0. < star.pos.z {
                star.reset();
                changed = true;
            }
        }
        
        if changed {
            self.sort();
        }
    }
}