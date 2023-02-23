use std::{path::{Path, PathBuf}, time::{SystemTime}};

use genmesh::{Triangulate, Vertices};
use glium::backend::Facade;
use obj::{ObjData, SimplePolygon};

use super::renderer::{VertexAttr, ObjRenderer};

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

            // let obj_source = read_to_string(path).unwrap();
            // let wavefront_objs = wavefront_obj::obj::parse(obj_source).unwrap();

            let objs = obj::Obj::load(path)?;
            // objs.data.
            // // objs.

            let (verts, indices) = vertices_and_indices(objs.data);

            // let mesh = MeshBuilder::new()
            //     .with_positions(verts.iter().flat_map(|v|v.position.iter()).map(|f| *f as f64).collect_vec())
            //     .with_indices(indices)
            //     .build().unwrap();

            // mesh.

            renderer.set_tri_data(facade, &verts, &indices);

            self.cur_file = Some(path.to_path_buf());
        }

        Ok(())
    }
}

fn vertices_and_indices(objs: ObjData) -> (Vec<VertexAttr>, Vec<u32>) {
    let positions = objs.position.iter().cloned().map(VertexAttr::new).collect();

    let indices = objs.objects.iter().flat_map(|obj| {
        obj.groups.iter().flat_map(|group| {
            group.polys.iter().cloned().map(SimplePolygon::into_genmesh)
        })
    })
        .triangulate()
        .vertices()
        .map(|index| index.0 as u32)
        .collect();

    (positions, indices)
}