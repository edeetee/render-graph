mod editor;
mod egui_glium;
mod tree_view;
mod widgets;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    egui_glium::main();

    Ok(())
}
