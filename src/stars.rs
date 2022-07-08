use std::iter;

// use kiss3d::nalgebra::Vector3;
use nannou::{rand::random_range, prelude::*};

//TODO: perspective camera?

#[derive(Debug, Default)]
pub struct Star{
    //position defined as m
    pub pos: Vec3,
    // pub tx: Mat4,
    //velocity defined as m/s
    pub vel: Vec3,
    //radius to draw
    pub radius: f32
}

// type Vec3 = Vec3;

impl Star{
    const SPEED: f32 = 100.0;

    fn new() -> Self {
        Self::default()
    }

    fn new_rand() -> Self {
       let mut new_star = Self::new();
       new_star.reset();
       new_star
    }

    fn update(&mut self, seconds: f32) {
        let delta = self.vel*seconds*Self::SPEED;
        
        self.pos += delta;
        // self.tx.transform_point3(delta);
        // self.vel = (self.vel*10. + Self::rand_vel()).normalize();
    }

    fn reset(&mut self) {
        self.reset_pos();
        self.reset_vel();
        self.reset_radius();
    }

    fn reset_pos(&mut self) {
        // self.tx
        self.pos.x = random_range(-POS_OFFSETXY, POS_OFFSETXY);
        self.pos.y = random_range(-POS_OFFSETXY, POS_OFFSETXY);
        self.pos.z = random_range(-1000., -500.);
    }

    fn reset_vel(&mut self) {
        self.vel.x = random_range(-VEL_OFFSET, VEL_OFFSET);
        self.vel.y = random_range(-VEL_OFFSET, VEL_OFFSET);
        self.vel.z = 1.;

        self.vel = self.vel.normalize();
    }

    fn reset_radius(&mut self) { 
        self.radius = random_range(0.5, 5.) 
    }
}

const VEL_OFFSET: f32 = 0.2;
const POS_OFFSETXY: f32 = 10.;


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
        }
    }
}