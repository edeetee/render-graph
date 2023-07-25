use std::{path::{Path, PathBuf}, time::{SystemTime}};

use genmesh::{Triangulate, Vertices, LruIndexer, Indexer};
use glium::backend::Facade;
use obj::{ObjData, SimplePolygon};

use super::renderer::{PosVertex, ObjRenderer, PosNormVertex, Data};

pub struct ObjLoader {
    cur_file: Option<PathBuf>,
    modified: SystemTime
}

impl ObjLoader {
    pub fn new() -> Self {
        Self {
            cur_file: None,
            modified: SystemTime::now()
        }
    }

    pub fn load_if_changed(&mut self, facade: &impl Facade, path: &Path, renderer: &mut ObjRenderer) -> Result<(), anyhow::Error> {
        let last_modified = path.metadata().unwrap().modified().unwrap();

        let do_load = match &self.cur_file {
            None => true,
            Some(cur_file) => {
                if let Ok(diff) = last_modified.duration_since(self.modified) {
                    10 < diff.as_millis()
                } else {
                    cur_file != path
                }
            }
        };

        if do_load {
            println!("Updating obj from {path:?}");

            let objs = obj::Obj::load(path)?;

            let data = vertices_and_indices(objs.data);

            renderer.update_data(facade, data);

            self.cur_file = Some(path.to_path_buf());
        }

        Ok(())
    }
}



fn vertices_and_indices(objs: ObjData) -> Data {
    let ObjData { 
        position, 
        texture: _, 
        normal, 
        objects, 
        material_libs: _ 
    } = objs;

    let indices = objects.iter()
        .flat_map(|obj| {
            obj.groups.iter().flat_map(|group| {
                group.polys.iter().cloned().map(SimplePolygon::into_genmesh)
            })
        })
        .triangulate()
        .vertices();

    // let positions = position.into_iter();

    if normal.is_empty() {
        Data::Pos(
            position.into_iter().map(PosVertex::new).collect(), 
            indices.map(|index|index.0 as u32).collect()
        )
    } else {
        let mut vertices: Vec<PosNormVertex> = vec![];
        let mut lru = LruIndexer::new(8, |_,b| {
            vertices.push(PosNormVertex::from(b));
        });

        let indices = indices.map(|index| {
            let vertex: genmesh::Vertex = genmesh::Vertex{
                pos: position[index.0].into(),
                normal: index.2.map(|index| normal[index]).unwrap_or([0.0,1.0,0.0]).into()
            }.into();

            lru.index(vertex) as u32
        }).collect();
        

        // todo!()
        Data::PosNorm(
            vertices, indices
        )
    }
}