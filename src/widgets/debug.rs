use std::ops::Deref;
use super::style::custom_dark_light_mode_switch;

pub fn debug_options(ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.set_clip_rect(ctx.available_rect());
    custom_dark_light_mode_switch(ui);

    let style = ctx.style();
    let mut debug = style.debug;
    debug.ui(ui);

    if debug != style.debug {
        let mut style = style.deref().clone();
        style.debug = debug;    
        ui.ctx().set_style(style);
    }
}