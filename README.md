# Shader graph renderer

A node graph renderer written in rust utilising [egui_node_graph](https://github.com/setzer22/egui_node_graph) for the graph view.

I am developing this for the use case of live generative visual performances, where I want to do performant, intuitive experimentation without crashes.

![screenshot](media/screenshot.jpg)

I come from Touchdesigner, a mature node graph system that has a tendency to crash when you push its boundaries.

## Features
- Spout (Windows texture sharing)
- ISF shader support
    - Hot reloading
- Texture sharing
- Obj file render

## Contributing

API is very much unstable at the moment. If you are interested in contributing, please make an issue and so I can stabilise the api surface required.

---

## TODO
- Make systems to define easy way to have common types across nodes so that it works nicely with rust Into<T> system and makes it easy to use graph features/types modularly and simply.
- Transparent windows https://ecode.dev/transparent-framebuffer-borderless-window-using-glfw/
- Only use Srgb textures for visible nodes
- Extend egui_node_graph for zooming etc
- Dependency resolution ala touchdesigner (only cook what is required)
- Hot reloading rust code
- Bevy / rend3 integration
- Full ISF spec
- Serialization of the graph / files etc
- Animation of parameters
- ISF Meta ops (more complex interface for handling racks + isf standard effects/transitions)
- Continue reducing dependencies between structs
- Support an SDF workflow
- Support different texture sizes (options per node)
- Sub shader input for obj render