# Shader graph rendering
- Minimal
- Performant
- Extendable
- Standardized

Touchdesigner and other node graph programming environments have the tendency to abstract the underlying code away. I want a system that allows the underlying code and graph representation to work together logically, performant and safely.

## Philosophy
- Performant (aim to be most performant general purpose extendable node graph renderer)
- Clear
    - Clear and minimal interface for nodes
    - Easy to imagine process
    - Good for beginner rustaceans
    - Thin (low abstraction)
- Extendable (Everything can be done at compile or during interactive session with safety)
- Stable
    - Rust safety
    - Mature api development

## TODO
- Make systems to define easy way to have common types across nodes so that it works nicely with rust Into<T> system and makes it easy to use graph features/types modularly and simply.
-Support transparent windows for pretty & clean testing https://ecode.dev/transparent-framebuffer-borderless-window-using-glfw/
- Only use Srgb textures for visible nodes
- Extend egui_node_graph for zooming etc
- Dependency graph ala touchdesigner
- Hot reloading
- Only use as many textures as required (min 2 required as sampler + render target) per node
- Investigate bevy integration