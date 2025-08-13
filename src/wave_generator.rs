use std::default;
use std::f32::consts::LN_2;

use godot::classes::rendering_device::{DataFormat, StorageBufferUsage, TextureUsageBits};
use godot::prelude::*;
use godot::classes::{Node, RdTextureView, RenderingServer};

use crate::rendering_context::{Descriptor, RenderingContext};
const G : f32 = 9.81;
const DEPTH: f32 = 20.0;
const SPECTRUM: usize = 0;
const 
#[derive(GodotClass)]
#[class(base=Node, init)]
struct WaveGenerator {
    map_size: i32,
    context: Option<Gd<RenderingContext>>,
    pipelines: Dictionary,
    descriptors: Vec<Descriptor>,
    base: Base<Node>
}
// #[godot_api]
// impl INode for WaveGenerator {

// }
impl WaveGenerator {
    fn init_gpu(&mut self, num_cascades: u32){
        // Device/Shader Creation
        if self.context == None {
            // self.context = Some(RenderingContext::new_gd());
            let mut temp_context = RenderingContext::new_gd();
            temp_context.bind_mut().initialize(RenderingServer::singleton().get_rendering_device());
            self.context = Some(temp_context);
        }
        let mut context = self.context.as_mut().expect("Context was None").bind_mut();
        let spectrum_compute_shader = context.load_shader("../shaders/compute/spectrum_compute.glsl".to_string());
        let fft_butterfly_shader = context.load_shader("../shaders/compute/fft_butterfly.glsl".to_string());
        let spectrum_modulate_shader = context.load_shader("../shaders/compute/spectrum_modulate.glsl".to_string());
        let fft_compute_shader = context.load_shader("../shaders/compute/fft_compute.glsl".to_string());
        let transpose_shader = context.load_shader("../shaders/compute/transpose.glsl".to_string());
        let fft_unpack_shader = context.load_shader("../shaders/compute/fft_unpack.glsl".to_string());
        let dims: Vector2i = Vector2i { x: self.map_size, y: self.map_size };
        let num_fft_stages : i32 = ((self.map_size as f32).ln() / LN_2).floor() as i32;

        // Prepare Descriptors:
        self.descriptors.push(Gd::from_object(
            context.create_texture(
                dims, DataFormat::R32G32B32A32_SFLOAT, 
                TextureUsageBits::STORAGE_BIT | TextureUsageBits::CAN_COPY_FROM_BIT, 
                num_cascades, 
                RdTextureView::new_gd(), 
                Array::new()
            )
        ));
        self.descriptors.set("butterfly_factors", Gd::from_object(
            context.create_storage_buffer(
                (num_fft_stages * self.map_size * 4 * 4) as usize, 
                PackedByteArray::new(), 
                StorageBufferUsage::DISPATCH_INDIRECT
            )
        ));
        self.descriptors.set("fft_buffer", Gd::from_object(
            context.create_storage_buffer(
                (num_cascades as i32 * self.map_size * self.map_size * 4 * 2 * 2 * 4) as usize, 
                PackedByteArray::new(), 
                StorageBufferUsage::DISPATCH_INDIRECT
            )
        ));
        self.descriptors.set("displacement_map", Gd::from_object(
            context.create_texture(
                dims, 
                DataFormat::R16G16B16A16_SFLOAT, 
                TextureUsageBits::STORAGE_BIT | TextureUsageBits::SAMPLING_BIT | TextureUsageBits::CAN_UPDATE_BIT, 
                num_cascades, 
                RdTextureView::new_gd(), 
                Array::new()
            )
        ));
        self.descriptors.set("normal_map", Gd::from_object(
            context.create_texture(
                dims, 
                DataFormat::R16G16B16A16_SFLOAT, 
                TextureUsageBits::STORAGE_BIT | TextureUsageBits::SAMPLING_BIT | TextureUsageBits::CAN_UPDATE_BIT, 
                num_cascades, 
                RdTextureView::new_gd(), 
                Array::new()
            )
        ));

        let spectrum_set = context.create_descriptor_set(self.descriptors.at("spectrum"), spectrum_compute_shader, descriptor_set_index)

    }
}