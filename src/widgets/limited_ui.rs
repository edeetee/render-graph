use egui::{DragValue};

pub enum UiLimit<T> {
    Clamp(Option<T>, Option<T>),
    Loop(T, T),
    None
}

impl <T> UiLimit<T> {
    fn min(&self) -> Option<&T> {
        match self {
            UiLimit::Clamp(min, _) => min.as_ref(),
            UiLimit::Loop(min, _) => Some(min),
            UiLimit::None => None,
        }
    }

    fn max(&self) -> Option<&T> {
        match self {
            UiLimit::Clamp(_, max) => max.as_ref(),
            UiLimit::Loop(_, max) => Some(max),
            UiLimit::None => None,
        }
    }
}

pub fn horizontal_drags<const A: usize>(
    ui: &mut egui::Ui, 
    labels: &[&str; A],
    limits: UiLimit<&[f32; A]>,
    values: &mut [f32; A],
) -> egui::InnerResponse<bool> {

    ui.horizontal(|ui| {
        let mut any_changed = false;

        for i in 0..A {
            // let range = &ranges[i];
            // let value = &mut values[i];
            let label = labels[i];

            ui.label(label);

            let min = limits.min().map(|min| min[i]);
            let max = limits.max().map(|max| max[i]);

            let speed =  match (min, max) {
                (Some(min), Some(max)) => 0.01 * (max - min).abs(),
                _ => 0.1
            };

            let drag_value_ui = DragValue::new(&mut values[i])
                .speed(speed);

            if ui.add(drag_value_ui).changed() {
                any_changed = true;
            }

            match limits {
                UiLimit::Loop(min, max) => {
                    let sum = (max[i] - min[i]).abs();
    
                    let mut temp_val = values[i];
    
                    //center at 0
                    temp_val -= min[i];
                    temp_val %= sum;
                    temp_val += min[i];
    
                    values[i] = temp_val;
                },

                UiLimit::Clamp(_, _) => {
                    if let Some(min) = min {
                        values[i] = values[i].max(min);
                    }
        
                    if let Some(max) = max {
                        values[i] = values[i].min(max);
                    }
                },
                UiLimit::None => {},
            }
        }

        any_changed
    })
}