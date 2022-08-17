# Shader graph rendering
I'm trying to make a super minimal shader graph for rust. It is build of glium so that it works on legacy devices (raspberry pi etc)
I use egui_node_graph for the node graph

## TODO
- Make systems to define easy way to have common types across nodes so that it works nicely with rust Into<T> system and makes it easy to use graph features/types modularly and simply.