use std::{iter};

// use kiss3d::nalgebra::Vector3;
use glam::Vec3;
use rand::{prelude::*, distributions::uniform::{SampleUniform, SampleRange}};

//TODO: perspective camera?

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
    // pub color: Hsv,
}

fn random_range<T,R>(range: R) -> T
where
    T: SampleUniform,
    R: SampleRange<T>, 
{
    rand::thread_rng().gen_range(range)
}

// type Vec3 = Vec3;

impl Star{
    // fn new() -> Self {
    //     Self::default()
    // }

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
        // self.rand_color();
    }

    fn rand_pos(&mut self) {
        // self.tx
        self.pos.x = random_range(-POS_OFFSETXY..POS_OFFSETXY);
        self.pos.y = random_range(-POS_OFFSETXY..POS_OFFSETXY);
        self.pos.z = random_range(-10f32..-0.5f32);
    }

    fn rand_vel(&mut self) {
        self.vel.x = random_range(-VEL_OFFSET..VEL_OFFSET);
        self.vel.y = random_range(-VEL_OFFSET..VEL_OFFSET);
        self.vel.z = 1.;

        self.vel = self.vel.normalize();
    }

    // fn rand_color(&mut self){
    //     self.color.hue = random_range(-180f32..180f32).into();
    //     self.color.saturation = random_range(0f32..0.5f32);
    //     self.color.value = 1.0;
    //     // self.color.r = rand::thread_rng().gen_range(COLOR_RANGE);
    //     // self.color.g = rand::thread_rng().gen_range(COLOR_RANGE);
    //     // self.color.b = rand::thread_rng().gen_range(COLOR_RANGE);
    // }

    fn rand_radius(&mut self) { 
        self.radius = random_range(0.5f32..1f32) 
    }
}

const VEL_OFFSET: f32 = 0.2;
const POS_OFFSETXY: f32 = 1.;


pub struct Stars {
    stars: Vec<Star>
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