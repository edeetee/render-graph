use std::{path::{Path, PathBuf}, time::{SystemTime}, fs::read_to_string};

use genmesh::{Triangle, Quad, Triangulate, Vertices};
use glium::backend::Facade;
use itertools::Itertools;
use obj::{ObjData, SimplePolygon, IndexTuple};
use tri_mesh::{MeshBuilder};
use wavefront_obj::obj::ObjSet;

use crate::obj_shader::renderer::vertices_from_mesh;

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

// fn tri_data_from_wavefront(objs: ObjSet) -> (Vec<VertexAttr>, Vec<u32>) {
//     let mut positions = Vec::new();
//     let mut indices = Vec::new();

//     for (_i, obj) in objs.objects.iter().enumerate() { // Objects consisting of several meshes with different materials
//         if obj.vertices.is_empty() || 16 <= obj.vertices.len() {
//             println!("- obj{}: {}v {}g", obj.name, obj.vertices.len(), obj.geometry.len());
//             continue;
//         } else {
//             println!("+ obj{}: {}v {}g {}t", obj.name, obj.vertices.len(), obj.geometry.len(), obj.tex_vertices.len());
//         }

//         let start_index = positions.len();

//         positions.extend(obj.vertices.iter()
//             .map(|v| {
//                 VertexAttr {
//                     position: [v.x as f32, -v.y as f32, v.z as f32]
//                 }
//             })
//         );

//         // obj.tex_vertices

//         for geo in &obj.geometry {
//             //index group per geometry
//             let geo_indices = geo.shapes.iter().flat_map(|primitive| { // All triangles with same material
//                 match primitive.primitive {
//                     wavefront_obj::obj::Primitive::Triangle(a, b, c) => {
//                         vec![
//                             (start_index + a.0) as u32,
//                             (start_index + b.0) as u32,
//                             (start_index + c.0) as u32
//                         ].into_iter()
//                     },
//                     _ => vec![].into_iter()
//                 }
//             });
//             indices.extend(geo_indices);
//         }
//     }

//     (positions, indices)

//     // (positions, indices)
// }