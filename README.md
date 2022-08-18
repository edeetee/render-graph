# Shader graph rendering
I'm trying to make a super minimal shader graph for rust. It is build of glium so that it works on legacy devices (raspberry pi etc)
I use egui_node_graph for the node graph

Touchdesigner and other node graph programming environments have the tendency to abstract the underlying code away. I want a system that allows the underlying code and graph representation to work together logically and performant.

## TODO
- Make systems to define easy way to have common types across nodes so that it works nicely with rust Into<T> system and makes it easy to use graph features/types modularly and simply.