use std::{
    fmt::Display,
    fs::read_dir,
    path::{Path, PathBuf}, rc::Rc,
};

use egui::{Widget, Rect, Stroke, Color32, Button, WidgetText, RichText};
use egui_glium::EguiGlium;
use glium::{backend::Facade, Surface, uniforms::{Uniforms, AsUniformValue}};
use itertools::Itertools;
use serde::Serialize;
use slotmap::SlotMap;
use try_utils::inner;

use crate::{
    common::connections::{ConnectionType, InputDef},
    graph::node_shader::{NodeShader},
    isf::meta::{default_isf_path, IsfInfo},
    textures::{ui::UiTexture, TextureManager},
    tree_view::{RefWidget, Tree},
};

use crate::graph::node_types::NodeType;

#[derive(Debug, Serialize)]
struct FilterState {
    image_inputs: bool,
    no_image_inputs: bool,
    text: String,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            image_inputs: false,
            no_image_inputs: false,
            text: String::new(),
        }
    }
}

impl FilterState {
    fn filter_item(&self, item: &LeafItem) -> bool {
        let text_pass = self.text.is_empty()
            || item
                .ty
                .get_name()
                .to_lowercase()
                .contains(&self.text.to_lowercase());

        let image_input_pass = {
            let has_inputs = item
                .ty
                .get_input_types()
                .iter()
                .any(|x| x.ty == ConnectionType::Texture2D);

            (!self.image_inputs || has_inputs) && (!self.no_image_inputs || !has_inputs)
        };

        text_pass && image_input_pass
    }
}

///Holds the data for the tree vi
pub struct TreeState {
    filter: FilterState,
    pub trees: Vec<Tree<LeafIndex, BranchIndex>>,
    pub branches: SlotMap<BranchIndex, BranchItem>,
    pub leaves: SlotMap<LeafIndex, LeafItem>,
}

impl Default for TreeState {
    fn default() -> Self {
        let mut branches = SlotMap::default();
        let mut leaves = SlotMap::default();

        let default_nodes = NodeType::get_builtin()
            .into_iter()
            .map(LeafItem::new)
            .map(|leaf| leaves.insert(leaf))
            .map(Tree::Leaf)
            .collect_vec();

        let cargo_shaders = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
        let default_shaders = default_isf_path();

        let isf_nodes = vec![cargo_shaders.as_ref(), default_shaders.as_ref()]
            .into_iter()
            .filter_map(partial!(load_isf_tree => _, &mut leaves, &mut branches));

        Self {
            trees: default_nodes.into_iter().chain(isf_nodes).collect(),
            filter: FilterState::default(),
            branches,
            leaves
        }
    }
}

pub struct TreeDrawResult {
    pub clicked: Option<LeafIndex>,
    pub in_view: Vec<LeafIndex>,
}

impl TreeState {
    /**
     * returns the selected item
     */
    pub fn draw(&mut self, ui: &mut egui::Ui) -> TreeDrawResult {
        let mut clicked_leaf = None;
        let mut search_changed = false;

        ui.heading("Node Types");

        let text_edit = ui.text_edit_singleline(&mut self.filter.text);
        // text_edit.request_focus();

        search_changed |= text_edit.changed();
        ui.horizontal(|ui| {
            ui.label("Image In");
            search_changed |= ui
                .toggle_value(&mut self.filter.image_inputs, "Some")
                .clicked();
            search_changed |= ui
                .toggle_value(&mut self.filter.no_image_inputs, "None")
                .clicked();
        });

        let open_state = if !search_changed {
            None
        } else if self.filter.text.is_empty() {
            None
        } else {
            Some(true)
        };

        let mut leaves_in_view = vec![];

        for tree in &mut self.trees {
            if search_changed {
                tree.map_leaf(&mut |item| {
                    let item = &mut self.leaves[item];
                    item.visible = self.filter.filter_item(item);
                });
            }

            tree.draw(ui, open_state, &mut |ui, leaf_index| {
                let leaf = &mut self.leaves[leaf_index];

                if leaf.visible {
                    let resp = leaf.ui(ui);

                    // let available_rect = ui.available_rect_before_wrap();

                    if resp.clicked() {
                        clicked_leaf = Some(leaf_index);
                    }

                    if ui.is_rect_visible(resp.rect) {
                        leaves_in_view.push(leaf_index);
                    }
                }
            }, &mut |branch_index| self.branches[branch_index].name.clone());
        }

        

        // clicked_leaf

        TreeDrawResult { clicked: clicked_leaf, in_view: leaves_in_view }
    }
}

pub struct LeafItem {
    visible: bool,
    pub ty: NodeType,
    //some(ok) if loaded
    //some(err) if failed to load
    //none if not loaded yet
    pub instance: Option<anyhow::Result<(NodeShader, UiTexture)>>,
}

struct LeafTempUniforms<'a> {
    pub input_tex: Option<&'a glium::Texture2d>,
    pub inputs: &'a [InputDef],
}

impl <'b> Uniforms for LeafTempUniforms<'b> {
    fn visit_values<'a, F: FnMut(&str, glium::uniforms::UniformValue<'a>)>(&'a self, mut output: F) {
        for input in self.inputs {
            let shader_input = if input.ty == ConnectionType::Texture2D {
                self.input_tex.as_ref().map(|tex|tex.as_uniform_value())
            } else {
                input.value.as_shader_input()
            };

            if let Some(uniform_value) = shader_input {
                output(&input.name, uniform_value);
            }
        }
    }
}

impl LeafItem {
    fn new(ty: NodeType) -> Self {
        Self {
            visible: true,
            ty,
            instance: None,
        }
    }

    pub fn render(&mut self, facade: &impl Facade, egui_glium: &mut EguiGlium, texture_manager: &mut TextureManager, input_tex: Option<&glium::Texture2d>) {
        if self.instance.is_none() {
            if let Some(shader) = NodeShader::new(&self.ty, facade) {
                self.instance = Some(shader.map(|shader| {
                    let img = UiTexture::new(facade, egui_glium, (LEAF_RENDER_WIDTH, LEAF_RENDER_WIDTH));
                    (shader, img)
                }));
            }
        }

        if let Some(Ok((shader, img))) = &mut self.instance {
            let inputs = self.ty.get_input_types();

            let uniforms = LeafTempUniforms{
                input_tex,
                inputs: &inputs
            };

            if let Ok(output) = shader.render(facade, texture_manager, uniforms) {
                img.copy_from(facade, &output.as_surface());
            } else {
                img.framebuffer(facade).unwrap().clear_color(1.0, 0.0, 0.0, 1.0);
            }
        }
    }

    fn new_from_isf(isf_path: PathBuf) -> Option<Self> {
        let info = IsfInfo::new_from_path(&isf_path).ok()?;

        Some(Self::new(NodeType::Isf { info }))
    }
}

const LEAF_RENDER_WIDTH: u32 = 64;

impl Widget for &LeafItem {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let all_width = LEAF_RENDER_WIDTH as f32;
        let all_size = [all_width; 2];
        let (all_rect, resp) = ui.allocate_exact_size(all_size.into(), egui::Sense::click());

        let inner_resp = egui::Frame::none()
            .stroke(Stroke::new(1.0, if resp.hovered() {Color32::WHITE} else {Color32::GRAY}))
            .show(ui, |ui|{

                if let Some(Ok((_,tex))) = &self.instance {
                    let (width, height) = tex.size();
                    let max = width.max(height);
                    let img_size = [width as f32, height as f32].map(|x|(x as f32) / (max as f32) * all_width);
                    
                    ui.put(all_rect, egui::Image::new(tex.id(), img_size));
                }
                ui.put(all_rect, egui::Label::new(RichText::new(self.to_string()).color(ui.visuals().text_color())));
                
            });

        inner_resp.response.union(resp)
    }
}

impl Display for LeafItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)
    }
}

pub struct BranchItem {
    pub name: String,
}

impl Display for BranchItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

slotmap::new_key_type! {
    pub struct LeafIndex;
}

slotmap::new_key_type! {
    pub struct BranchIndex;
}



fn load_isf_tree(path: &Path, leaves: &mut SlotMap<LeafIndex, LeafItem>, branches: &mut SlotMap<BranchIndex,BranchItem>) -> Option<Tree<LeafIndex, BranchIndex>> {
    if path.is_dir() {
        let branch_inner = read_dir(path)
            .unwrap()
            .into_iter()
            .filter_map(|dir| load_isf_tree(&dir.unwrap().path(), leaves, branches))
            .collect();

        let info = BranchItem {
            name: path.file_name().unwrap().to_str().unwrap().to_string(),
        };

        Some(Tree::Branch(branches.insert(info), branch_inner))
    } else {
        let info = LeafItem::new_from_isf(path.clone().to_path_buf())?;
        Some(Tree::Leaf(leaves.insert(info)))
    }
}
