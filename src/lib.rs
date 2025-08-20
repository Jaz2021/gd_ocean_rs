use godot::prelude::*;
mod ocean;
mod wave_cascade_parameters;
mod wave_generator;
mod rendering_context;
struct GDOcean;

#[gdextension]
unsafe impl ExtensionLibrary for GDOcean {}
