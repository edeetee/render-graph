use egui::{Color32, Ui, Visuals};

trait SelfCall: Sized {
    fn apply(self, f: fn(Self) -> Self) -> Self {
        f(self)
    }
    fn mutate(&mut self, f: impl Fn(&mut Self) -> Self) -> &mut Self {
        *self = f(self);
        self
    }
}

impl<T> SelfCall for T {}

pub fn custom_dark_light_mode_switch(ui: &mut Ui) {
    let style: egui::Style = (*ui.ctx().style()).clone();
    let new_visuals = style.visuals.light_dark_small_toggle_button(ui);
    if let Some(visuals) = new_visuals {
        ui.ctx().set_visuals(visuals.apply(modifications));
    }
}

pub fn modifications(mut visuals: Visuals) -> Visuals {
    // visuals.widgets.noninteractive.bg_fill = Color32::from_black_alpha(200);
    visuals
        .widgets
        .noninteractive
        .bg_fill
        .mutate(|c: &mut Color32| Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 220));
    visuals
}

pub fn custom_visuals() -> Visuals {
    egui::Visuals::dark().apply(modifications)
}
