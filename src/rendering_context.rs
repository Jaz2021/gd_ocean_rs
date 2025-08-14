use std::any::{Any, TypeId};

use godot::classes::notify::ObjectNotification;
use godot::classes::rendering_device::{self, DataFormat, TextureType, TextureUsageBits, UniformType};
use godot::prelude::*;
use godot::classes::{RdShaderFile, RdTextureFormat, RdTextureView, RdUniform, RenderingDevice, RenderingServer, Resource, ShaderMaterial};
#[derive(GodotClass)]
#[class(base=Resource)]
pub struct RenderingContext {
    device: Option<Gd<RenderingDevice>>,
    deletion_queue: DeletionQueue,
    shader_cache: Dictionary,
    needs_sync: bool,
    base: Base<Resource>
}
#[godot_api]
impl IResource for RenderingContext {
    fn init(base: Base<Resource>) -> Self {
        godot_print!("Hello world!");
        // let device = RenderingServer::singleton().create_local_rendering_device();
        Self {
            device: None,
            deletion_queue: DeletionQueue { queue: Array::new() },
            shader_cache: Dictionary::new(),
            needs_sync: false,
            base
        }
    }
    fn on_notification(&mut self, what : ObjectNotification){
        match what{
            ObjectNotification::PREDELETE => {
                // All resources must be freed
                // let mut dev = self.device;
                if let Some(held) = self.device.take() {
                    self.deletion_queue.flush(&mut held.clone());
                    self.shader_cache.clear();
                    let rendering_device = RenderingServer::singleton().get_rendering_device();
                    match rendering_device {
                        Some(render) => {
                            if render != held {
                                // As long as the current rendering device isn't the one we are connected to, kill it
                                held.free(); // Now we own `held` so we can consume it
                            }
                        }
                        None => {}
                    }
                }
                // match self.device{
                //     Some(ref mut held) => {
                //         self.deletion_queue.flush(held);
                //         self.shader_cache.clear();

                //         let rendering_device = RenderingServer::singleton().get_rendering_device();
                //         match rendering_device {
                //             Some(render) => {
                //                 if render != *held {
                //                     // As long as the current rendering device isn't the one we are connected to, kill it
                //                     held.free();
                //                 }
                //             }
                //             None => {}
                //         }
                //     }
                //     None => {
                //         // Do nothing
                //     }
                // }
            }
            _ => {
                // Do nothing
            }
        }
    }
}
#[godot_api]
impl RenderingContext {
    pub fn initialize(&mut self, device: Option<Gd<RenderingDevice>>){
        if device == None {
            self.device = RenderingServer::singleton().create_local_rendering_device();
        } else {
            self.device = device;
        }
    }
    #[func]
    fn submit(&mut self){
        self.device.as_mut().unwrap().submit();
        self.needs_sync = true;
    }
    #[func]
    fn sync(&mut self){
        self.device.as_mut().unwrap().sync();
        self.needs_sync = false;
    }
    pub fn compute_list_begin(&mut self) -> i64 {
        return self.device.as_mut().unwrap().compute_list_begin();
    }
    pub fn compute_list_end(&mut self) {
        self.device.as_mut().unwrap().compute_list_end();
    }
    pub fn compute_list_add_buffer(&mut self, compute_list: i64){
        self.device.as_mut().unwrap().compute_list_add_barrier(compute_list);
    }
    #[func]
    pub fn load_shader(&mut self, path: String) -> Rid {
        if !self.shader_cache.contains_key(path.as_str()){
            let shader_file = load::<RdShaderFile>(path.as_str());
            let shader_spirv = shader_file.get_spirv().expect(format!("{path} was not a valid shader file, get_spirv failed").as_str());
            self.deletion_queue.push(shader_spirv.get_rid());
            self.shader_cache.set(path.as_str(), shader_spirv.get_rid());
        }
        return self.shader_cache.get(path.as_str()).expect("Path was not a valid shaderMaterial or something else went wrong not sure").to();
    }
    // #[func]
    pub fn create_storage_buffer(&mut self, mut size: usize, mut data: PackedByteArray, usage: rendering_device::StorageBufferUsage) -> Descriptor {
        size = size.max(16);
        if size > data.len() {
            // let mut padding = PackedByteArray::new();
            // padding.resize(size - data.len());
            for _ in 0..(size - data.len()) {
                data.push(0u8);
            }
        }
        let mut buffer = self.device.as_mut().expect("Rendering context device is none").storage_buffer_create_ex((size as u32).max(data.len() as u32));
        buffer = buffer.data(&data);
        buffer = buffer.usage(usage);
        let rid = buffer.done();
        self.deletion_queue.push(rid);
        Descriptor { rid: rid, descriptor_type: UniformType::STORAGE_BUFFER }
    }
    pub fn create_uniform_buffer(&mut self, mut size: usize, mut data: PackedByteArray) -> Descriptor {
        size = size.max(16);
        if size > data.len() {
            for _ in 0..(size - data.len()) {
                data.push(0u8);
            }
        }
        let mut buffer = self.device.as_mut().expect("Rendering device is none").uniform_buffer_create_ex(size.max(data.len()) as u32);
        buffer = buffer.data(&data);
        let rid = buffer.done();
        self.deletion_queue.push(rid);
        Descriptor { rid: rid, descriptor_type: UniformType::UNIFORM_BUFFER }
    }
    pub fn create_texture(&mut self, dimensions: Vector2i, format: DataFormat, usage: TextureUsageBits, mut num_layers: u32, view: Gd<RdTextureView>, data: Array<PackedByteArray>) -> Descriptor{
        if num_layers < 1{
            num_layers = 1;
            // panic!("Num layers in create_texture less than 1");
            godot_print!("Num layers in create_texture was set to 0, should be at least 1. Defaulted to 1");
        }
        let mut texture_format = RdTextureFormat::new_gd();
        texture_format.set_array_layers(num_layers);
        texture_format.set_format(format);
        texture_format.set_width(dimensions.x as u32);
        texture_format.set_height(dimensions.y as u32);
        texture_format.set_texture_type(TextureType::TYPE_2D);
        texture_format.set_usage_bits(usage); 
        // Default RenderingDevice.TEXTURE_USAGE_SAMPLING_BIT | RenderingDevice.TEXTURE_USAGE_COLOR_ATTACHMENT_BIT | RenderingDevice.TEXTURE_USAGE_STORAGE_BIT | RenderingDevice.TEXTURE_USAGE_CAN_COPY_TO_BIT | RenderingDevice.TEXTURE_USAGE_CAN_COPY_FROM_BIT
        let texture = self.device.as_mut().expect("Rendering device is none").texture_create_ex(&texture_format, &view);
        let rid = texture.data(&data).done();
        self.deletion_queue.push(rid);
        Descriptor { rid: rid, descriptor_type: UniformType::IMAGE }
    }
    // ## Creates a descriptor set. The ordering of the provided descriptors matches the binding ordering
    // ## within the shader.
    // Seemingly the vector of descriptors was unnecessary so it is now a single descriptor instead
    pub fn create_descriptor_set(&mut self, descriptor:&Descriptor, shader: Rid, descriptor_set_index: u32) -> Rid{
        let mut uniforms: Array<Gd<RdUniform>> = Array::new();
        // for i in 0..descriptors.len() {
        let mut uniform = RdUniform::new_gd();
        uniform.set_uniform_type(descriptor.descriptor_type);
        uniform.set_binding(0 as i32);
        uniform.add_id(descriptor.rid);
        uniforms.push(&uniform);
        // }
        let rid = self.device.as_mut().expect("Rendering device is none").uniform_set_create(&uniforms, shader, descriptor_set_index);
        self.deletion_queue.push(rid);
        return rid;
    }
    pub fn create_descriptor_set_dual(&mut self, descriptor:&Descriptor, descriptor2: &Descriptor, shader: Rid, descriptor_set_index: u32) -> Rid{
        let mut uniforms: Array<Gd<RdUniform>> = Array::new();
        // for i in 0..descriptors.len() {
        let mut uniform = RdUniform::new_gd();
        uniform.set_uniform_type(descriptor.descriptor_type);
        uniform.set_binding(0 as i32);
        uniform.add_id(descriptor.rid);
        uniforms.push(&uniform);
        let mut uniform = RdUniform::new_gd();
        uniform.set_uniform_type(descriptor2.descriptor_type);
        uniform.set_binding(1 as i32);
        uniform.add_id(descriptor2.rid);
        uniforms.push(&uniform);
        // }
        let rid = self.device.as_mut().expect("Rendering device is none").uniform_set_create(&uniforms, shader, descriptor_set_index);
        self.deletion_queue.push(rid);
        return rid;
    }
    pub fn create_pipeline(
        &mut self, 
        block_dimensions: Vec<i32>, 
        descriptor_sets: Vec<Rid>, 
        shader: Rid
    ) -> Callable {
        // Create the pipeline using your deletion queue and device
        let pipeline = self.device.as_mut().expect("Rendering device is none").compute_pipeline_create(shader);
        self.deletion_queue.push(pipeline);
        
        
        // Create and return the Callable
        Callable::from_local_fn("pipeline_execute", move |args: &[&Variant]| -> Result<Variant, ()> {
            // Extract arguments from the Variant array
            let mut context = args.get(0)
                .and_then(|v| v.try_to::<Gd<RenderingContext>>().ok())
                .expect("First argument must be RenderingContext");
            
            let compute_list = args.get(1)
                .and_then(|v| v.try_to::<i64>().ok())
                .unwrap_or(0);
            
            let push_constant = args.get(2)
                .and_then(|v| v.try_to::<PackedByteArray>().ok())
                .unwrap_or_else(|| PackedByteArray::new());
            
            let descriptor_set_overwrites: Vec<Rid> = args.get(3)
                .and_then(|v| v.try_to::<VariantArray>().ok())
                .map(|arr| {
                    (0..arr.len())
                        .filter_map(|i| arr.at(i).try_to::<Rid>().ok())
                        .collect()
                })
                .unwrap_or_default();
            
            let block_dimensions_overwrite_buffer = args.get(4)
                .and_then(|v| v.try_to::<Rid>().ok());
            
            let block_dimensions_overwrite_buffer_byte_offset = args.get(5)
                .and_then(|v| v.try_to::<i64>().ok())
                .unwrap_or(0);
            
            // Execute the pipeline logic
            let mut context_bind = context.bind_mut();

            let mut device = context_bind.device.as_mut().expect("Rendering device was none");
            
            let sets = if descriptor_set_overwrites.is_empty() {
                &descriptor_sets
            } else {
                &descriptor_set_overwrites
            };
            
            // Assertions
            assert!(
                block_dimensions.len() == 3 || block_dimensions_overwrite_buffer.is_some(),
                "Must specify block dimensions or specify a dispatch indirect buffer!"
            );
            assert!(sets.len() >= 1, "Must specify at least one descriptor set!");
            
            // Bind pipeline and set push constants
            device.compute_list_bind_compute_pipeline(compute_list, pipeline);
            device.compute_list_set_push_constant(
                compute_list, 
                &push_constant, 
                push_constant.len() as u32
            );
            
            // Bind uniform sets
            for (i, &set) in sets.iter().enumerate() {
                device.compute_list_bind_uniform_set(compute_list, set, i as u32);
            }
            
            // Dispatch
            if let Some(buffer) = block_dimensions_overwrite_buffer {
                device.compute_list_dispatch_indirect(
                    compute_list, 
                    buffer, 
                    block_dimensions_overwrite_buffer_byte_offset as u32
                );
            } else {
                device.compute_list_dispatch(
                    compute_list,
                    block_dimensions[0] as u32,
                    block_dimensions[1] as u32,
                    block_dimensions[2] as u32,
                );
            }
            
            // Return void (empty Variant)
            Ok(Variant::nil())
        })
    }
    // ## Returns a [PackedFloat32Array] from the provided data, whose size is rounded up to the nearest
    // ## multiple of 16
    pub fn create_push_constant(data : &[Variant]) -> PackedByteArray{
        let packed_size: i32 = (data.len() * 4) as i32;
        if packed_size <= 128{
            panic!("Push constant size must be at most 128 bytes!");
        } 

        let padding = (packed_size as f32 / 16.0).ceil() as i32 * 16 - packed_size;
        let mut packed_data = PackedByteArray::new();
        if padding > 0 {
            packed_data.resize((packed_size + padding) as usize);
        } else {
            packed_data.resize(packed_size as usize);
        }
        packed_data.fill(0);
        for i in 0..data.len() {
            let d = &data[i];
            if d.type_id() == TypeId::of::<i32>(){
                _ = packed_data.encode_s32(i * 4, d.to());
            } else if d.type_id() == TypeId::of::<f32>(){
                _ = packed_data.encode_float(i * 4, d.to());
            }
        }
        return packed_data
    }
}
#[derive(GodotClass)]
#[class(no_init)]
pub struct Descriptor {
    rid: Rid,
    descriptor_type: UniformType
}
impl Default for Descriptor {
    fn default() -> Self {
        Self { rid: Rid::Invalid, descriptor_type: UniformType::STORAGE_BUFFER }
    }
}
struct DeletionQueue {
    queue: Array<Rid>,
}
impl DeletionQueue {
    pub fn push(&mut self, rid: Rid){
        self.queue.push(rid);
    }
    pub fn flush(&mut self, device: &mut Gd<RenderingDevice>){
        // work backwards in order of allocation when freeing resources
        let size = self.queue.len();
        for i in size - 1..0{
            match self.queue.get(i){
                Some(x) => {
                    if x.is_valid() {
                        device.free_rid(x);
                    } else {
                        continue;
                    }
                }
                None => {
                    continue;
                }
            }
        }
        self.queue.clear();
    }
    pub fn free_rid(&mut self, device: &mut Gd<RenderingDevice>, rid: Rid){
        let rid_idx = self.queue.find(rid, None);
        match rid_idx {
            Some(x) => {
                device.free_rid(rid);
                self.queue.remove(x);
            }
            None => {

            }
        }
    }
}
