use egui::DragValue;
use egui_node_graph::WidgetValueTrait;

use super::def::ValueTypes;

impl WidgetValueTrait for ValueTypes {
    type Response = ();

    fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) -> Vec<Self::Response> {
        match self {
            ValueTypes::Vec2 { value } => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(DragValue::new(&mut value[0]));
                    ui.label("y");
                    ui.add(DragValue::new(&mut value[1]));
                });
            }
            ValueTypes::Float { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(value));
                });
            }
            ValueTypes::None => {},
        }
        
        Vec::new()
    }
}

// TODO: draw the interface