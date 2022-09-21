use std::{path::Path, fs::File, time::SystemTime, io::Read};

use assets_manager::{AssetCache};


pub struct CachedFile{
    file: File,
    last_modification_time: SystemTime,
    text: String
}

impl CachedFile{
    pub fn new(mut file: File) -> Self {
        let mut text = String::new();
        file.read_to_string(&mut text);

        Self {
            last_modification_time: file.metadata().unwrap().modified().unwrap(),
            file,
            text
        }
    }

    pub fn read(&mut self) -> &String {
        let metadata = self.file.metadata().unwrap();
        let last_modification_time = metadata.modified().unwrap();

        if last_modification_time > self.last_modification_time {
            self.last_modification_time = last_modification_time;
            self.file.read_to_string(&mut self.text);
        }

        &self.text
    }

    // pub fn is_updated(&mut self) -> bool {
    //     let modification_time = self.file.metadata().unwrap().modified().unwrap();
    //     let has_been_modified = self.last_modification_time != modification_time;

    //     if has_been_modified {
    //         self.last_modification_time = modification_time;
    //     }

    //     has_been_modified
    // }
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref ASSETS: AssetCache = {
        let shader_file = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src\\modular_shader");

        println!("ASSETS DIR: {shader_file:?}");

        AssetCache::new(shader_file).unwrap()
    };
}

//An object that can render to a surface with parameters
// pub trait ModularShader<T>
// {
//     ///Draw to a surface
//     fn draw_to(&self, surface: &mut SimpleFrameBuffer<'_>, data: T) -> Result<(), DrawError>;
//     fn update(&mut self, _update: &ShaderUpdate) {}
// }

// pub enum ShaderUpdate {
//     Resolution([f32; 2]),
// }
