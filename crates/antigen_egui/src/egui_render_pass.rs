use antigen_wgpu::wgpu::{
    util::DeviceExt, Device, ShaderFlags, ShaderModuleDescriptor, ShaderSource, TextureFormat,
};
use bytemuck::{Pod, Zeroable};
use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    num::NonZeroU32,
};

/// Enum for selecting the right buffer type.
#[derive(Debug)]
enum BufferType {
    Uniform,
    Index,
    Vertex,
}

/// Information about the screen used for rendering.
pub struct ScreenDescriptor {
    /// Width of the window in physical pixel.
    pub physical_width: u32,
    /// Height of the window in physical pixel.
    pub physical_height: u32,
    /// HiDPI scale factor.
    pub scale_factor: f32,
}

impl ScreenDescriptor {
    fn logical_size(&self) -> (u32, u32) {
        let logical_width = self.physical_width as f32 / self.scale_factor;
        let logical_height = self.physical_height as f32 / self.scale_factor;
        (logical_width as u32, logical_height as u32)
    }
}

/// Uniform buffer used when rendering.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct UniformBuffer {
    screen_size: [f32; 2],
}

unsafe impl Pod for UniformBuffer {}

unsafe impl Zeroable for UniformBuffer {}

/// Wraps the buffers and includes additional information.
#[derive(Debug)]
struct SizedBuffer {
    buffer: antigen_wgpu::wgpu::Buffer,
    size: usize,
}

/// RenderPass to render a egui based GUI.
pub struct EguiRenderPass {
    render_pipeline: antigen_wgpu::wgpu::RenderPipeline,
    index_buffers: Vec<SizedBuffer>,
    vertex_buffers: Vec<SizedBuffer>,
    uniform_buffer: SizedBuffer,
    uniform_bind_group: antigen_wgpu::wgpu::BindGroup,
    texture_bind_group_layout: antigen_wgpu::wgpu::BindGroupLayout,
    texture_bind_group: Option<antigen_wgpu::wgpu::BindGroup>,
    texture_version: Option<u64>,
    next_user_texture_id: u64,
    pending_user_textures: Vec<(u64, egui::Texture)>,
    user_textures: Vec<Option<antigen_wgpu::wgpu::BindGroup>>,
}

impl Debug for EguiRenderPass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EguiRenderPass")
    }
}

impl Display for EguiRenderPass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EguiRenderPass")
    }
}

impl EguiRenderPass {
    /// Creates a new render pass to render an egui UI.
    pub async fn new(device: &Device, output_format: TextureFormat) -> Self {
        #[cfg(not(feature = "web"))]
        let vs_module = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("egui_vertex_shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader/egui_vert.wgsl"))),
            flags: ShaderFlags::all(),
        });

        #[cfg(feature = "web")]
        let vs_module = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("egui_vertex_shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader/egui_vert_web.wgsl"))),
            flags: ShaderFlags::all(),
        });

        let fs_module = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("egui_fragment_shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader/egui_frag.wgsl"))),
            flags: ShaderFlags::all(),
        });

        let uniform_buffer =
            device.create_buffer_init(&antigen_wgpu::wgpu::util::BufferInitDescriptor {
                label: Some("egui_uniform_buffer"),
                contents: bytemuck::cast_slice(&[UniformBuffer {
                    screen_size: [0.0, 0.0],
                }]),
                usage: antigen_wgpu::wgpu::BufferUsage::UNIFORM
                    | antigen_wgpu::wgpu::BufferUsage::COPY_DST,
            });
        let uniform_buffer = SizedBuffer {
            buffer: uniform_buffer,
            size: std::mem::size_of::<UniformBuffer>(),
        };

        let sampler = device.create_sampler(&antigen_wgpu::wgpu::SamplerDescriptor {
            label: Some("egui_texture_sampler"),
            mag_filter: antigen_wgpu::wgpu::FilterMode::Linear,
            min_filter: antigen_wgpu::wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&antigen_wgpu::wgpu::BindGroupLayoutDescriptor {
                label: Some("egui_uniform_bind_group_layout"),
                entries: &[
                    antigen_wgpu::wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: antigen_wgpu::wgpu::ShaderStage::VERTEX,
                        ty: antigen_wgpu::wgpu::BindingType::Buffer {
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            ty: antigen_wgpu::wgpu::BufferBindingType::Uniform,
                        },
                        count: None,
                    },
                    antigen_wgpu::wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: antigen_wgpu::wgpu::ShaderStage::FRAGMENT,
                        ty: antigen_wgpu::wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        count: None,
                    },
                ],
            });

        let uniform_bind_group =
            device.create_bind_group(&antigen_wgpu::wgpu::BindGroupDescriptor {
                label: Some("egui_uniform_bind_group"),
                layout: &uniform_bind_group_layout,
                entries: &[
                    antigen_wgpu::wgpu::BindGroupEntry {
                        binding: 0,
                        resource: antigen_wgpu::wgpu::BindingResource::Buffer(
                            antigen_wgpu::wgpu::BufferBinding {
                                buffer: &uniform_buffer.buffer,
                                offset: 0,
                                size: None,
                            },
                        ),
                    },
                    antigen_wgpu::wgpu::BindGroupEntry {
                        binding: 1,
                        resource: antigen_wgpu::wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&antigen_wgpu::wgpu::BindGroupLayoutDescriptor {
                label: Some("egui_texture_bind_group_layout"),
                entries: &[antigen_wgpu::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: antigen_wgpu::wgpu::ShaderStage::FRAGMENT,
                    ty: antigen_wgpu::wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: antigen_wgpu::wgpu::TextureSampleType::Float {
                            filterable: true,
                        },
                        view_dimension: antigen_wgpu::wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&antigen_wgpu::wgpu::PipelineLayoutDescriptor {
                label: Some("egui_pipeline_layout"),
                bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&antigen_wgpu::wgpu::RenderPipelineDescriptor {
            label: Some("egui_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: antigen_wgpu::wgpu::VertexState {
                entry_point: "main",
                module: &vs_module,
                buffers: &[antigen_wgpu::wgpu::VertexBufferLayout {
                    array_stride: 5 * 4,
                    step_mode: antigen_wgpu::wgpu::InputStepMode::Vertex,
                    // 0: vec2 position
                    // 1: vec2 texture coordinates
                    // 2: uint color
                    attributes: &antigen_wgpu::wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint32],
                }],
            },
            primitive: antigen_wgpu::wgpu::PrimitiveState {
                topology: antigen_wgpu::wgpu::PrimitiveTopology::TriangleList,
                clamp_depth: false,
                conservative: false,
                cull_mode: None,
                front_face: antigen_wgpu::wgpu::FrontFace::default(),
                polygon_mode: antigen_wgpu::wgpu::PolygonMode::default(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: antigen_wgpu::wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0,
            },

            fragment: Some(antigen_wgpu::wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[antigen_wgpu::wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(antigen_wgpu::wgpu::BlendState {
                        color: antigen_wgpu::wgpu::BlendComponent {
                            src_factor: antigen_wgpu::wgpu::BlendFactor::One,
                            dst_factor: antigen_wgpu::wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: antigen_wgpu::wgpu::BlendOperation::Add,
                        },
                        alpha: antigen_wgpu::wgpu::BlendComponent {
                            src_factor: antigen_wgpu::wgpu::BlendFactor::OneMinusDstAlpha,
                            dst_factor: antigen_wgpu::wgpu::BlendFactor::One,
                            operation: antigen_wgpu::wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: antigen_wgpu::wgpu::ColorWrite::ALL,
                }],
            }),
        });

        Self {
            render_pipeline,
            vertex_buffers: Vec::with_capacity(64),
            index_buffers: Vec::with_capacity(64),
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group_layout,
            texture_version: None,
            texture_bind_group: None,
            next_user_texture_id: 0,
            pending_user_textures: Vec::new(),
            user_textures: Vec::new(),
        }
    }

    /// Executes the egui render pass. When `clear_on_draw` is set, the output target will get cleared before writing to it.
    pub fn execute(
        &mut self,
        encoder: &mut antigen_wgpu::wgpu::CommandEncoder,
        color_attachment: &antigen_wgpu::wgpu::TextureView,
        paint_jobs: &[egui::paint::ClippedMesh],
        screen_descriptor: &ScreenDescriptor,
        clear_color: Option<antigen_wgpu::wgpu::Color>,
    ) {
        let load_operation = if let Some(color) = clear_color {
            antigen_wgpu::wgpu::LoadOp::Clear(color)
        } else {
            antigen_wgpu::wgpu::LoadOp::Load
        };

        let mut pass = encoder.begin_render_pass(&antigen_wgpu::wgpu::RenderPassDescriptor {
            color_attachments: &[antigen_wgpu::wgpu::RenderPassColorAttachment {
                view: color_attachment,
                resolve_target: None,
                ops: antigen_wgpu::wgpu::Operations {
                    load: load_operation,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
            label: Some("egui main render pass"),
        });
        pass.push_debug_group("egui_pass");
        pass.set_pipeline(&self.render_pipeline);

        pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        let scale_factor = screen_descriptor.scale_factor;
        let physical_width = screen_descriptor.physical_width;
        let physical_height = screen_descriptor.physical_height;

        for ((egui::ClippedMesh(clip_rect, mesh), vertex_buffer), index_buffer) in paint_jobs
            .iter()
            .zip(self.vertex_buffers.iter())
            .zip(self.index_buffers.iter())
        {
            // Transform clip rect to physical pixels.
            let clip_min_x = scale_factor * clip_rect.min.x;
            let clip_min_y = scale_factor * clip_rect.min.y;
            let clip_max_x = scale_factor * clip_rect.max.x;
            let clip_max_y = scale_factor * clip_rect.max.y;

            // Make sure clip rect can fit within an `u32`.
            let clip_min_x = clip_min_x.clamp(0.0, physical_width as f32);
            let clip_min_y = clip_min_y.clamp(0.0, physical_height as f32);
            let clip_max_x = clip_max_x.clamp(clip_min_x, physical_width as f32);
            let clip_max_y = clip_max_y.clamp(clip_min_y, physical_height as f32);

            let clip_min_x = clip_min_x.round() as u32;
            let clip_min_y = clip_min_y.round() as u32;
            let clip_max_x = clip_max_x.round() as u32;
            let clip_max_y = clip_max_y.round() as u32;

            let width = (clip_max_x - clip_min_x).max(1);
            let height = (clip_max_y - clip_min_y).max(1);

            {
                // clip scissor rectangle to target size
                let x = clip_min_x.min(physical_width);
                let y = clip_min_y.min(physical_height);
                let width = width.min(physical_width - x);
                let height = height.min(physical_height - y);

                // skip rendering with zero-sized clip areas
                if width == 0 || height == 0 {
                    continue;
                }

                pass.set_scissor_rect(x, y, width, height);
            }
            pass.set_bind_group(1, self.get_texture_bind_group(mesh.texture_id), &[]);

            pass.set_index_buffer(
                index_buffer.buffer.slice(..),
                antigen_wgpu::wgpu::IndexFormat::Uint32,
            );
            pass.set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
            pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
        }

        pass.pop_debug_group();
    }

    fn get_texture_bind_group(
        &self,
        texture_id: egui::TextureId,
    ) -> &antigen_wgpu::wgpu::BindGroup {
        match texture_id {
            egui::TextureId::Egui => self
                .texture_bind_group
                .as_ref()
                .expect("egui texture was not set before the first draw"),
            egui::TextureId::User(id) => {
                let id = id as usize;
                assert!(id < self.user_textures.len());
                &(self
                    .user_textures
                    .get(id)
                    .unwrap_or_else(|| panic!("user texture {} not found", id))
                    .as_ref()
                    .unwrap_or_else(|| panic!("user texture {} freed", id)))
            }
        }
    }

    /// Updates the texture used by egui for the fonts etc. Should be called before `execute()`.
    pub fn update_texture(
        &mut self,
        device: &antigen_wgpu::wgpu::Device,
        queue: &antigen_wgpu::wgpu::Queue,
        egui_texture: &egui::Texture,
    ) {
        // Don't update the texture if it hasn't changed.
        if self.texture_version == Some(egui_texture.version) {
            return;
        }
        // we need to convert the texture into rgba_srgb format
        let mut pixels: Vec<u8> = Vec::with_capacity(egui_texture.pixels.len() * 4);
        for srgba in egui_texture.srgba_pixels() {
            pixels.push(srgba.r());
            pixels.push(srgba.g());
            pixels.push(srgba.b());
            pixels.push(srgba.a());
        }
        let egui_texture = egui::Texture {
            version: egui_texture.version,
            width: egui_texture.width,
            height: egui_texture.height,
            pixels,
        };
        let bind_group = self.egui_texture_to_wgpu(device, queue, &egui_texture, "egui");

        self.texture_version = Some(egui_texture.version);
        self.texture_bind_group = Some(bind_group);
    }

    /// Updates the user textures that the app allocated. Should be called before `execute()`.
    pub fn update_user_textures(
        &mut self,
        device: &antigen_wgpu::wgpu::Device,
        queue: &antigen_wgpu::wgpu::Queue,
    ) {
        let pending_user_textures = std::mem::take(&mut self.pending_user_textures);
        for (id, texture) in pending_user_textures {
            let bind_group = self.egui_texture_to_wgpu(
                device,
                queue,
                &texture,
                format!("user_texture{}", id).as_str(),
            );
            self.user_textures.push(Some(bind_group));
        }
    }

    // Assumes egui_texture contains srgb data.
    // This does not match how egui::Texture is documented as of writing, but this is how it is used for user textures.
    fn egui_texture_to_wgpu(
        &self,
        device: &antigen_wgpu::wgpu::Device,
        queue: &antigen_wgpu::wgpu::Queue,
        egui_texture: &egui::Texture,
        label: &str,
    ) -> antigen_wgpu::wgpu::BindGroup {
        let size = antigen_wgpu::wgpu::Extent3d {
            width: egui_texture.width as u32,
            height: egui_texture.height as u32,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&antigen_wgpu::wgpu::TextureDescriptor {
            label: Some(format!("{}_texture", label).as_str()),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: antigen_wgpu::wgpu::TextureDimension::D2,
            format: antigen_wgpu::wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: antigen_wgpu::wgpu::TextureUsage::SAMPLED
                | antigen_wgpu::wgpu::TextureUsage::COPY_DST,
        });

        queue.write_texture(
            antigen_wgpu::wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: antigen_wgpu::wgpu::Origin3d::ZERO,
            },
            egui_texture.pixels.as_slice(),
            antigen_wgpu::wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(
                    (egui_texture.pixels.len() / egui_texture.height) as u32,
                ),
                rows_per_image: NonZeroU32::new(egui_texture.height as u32),
            },
            size,
        );

        let bind_group = device.create_bind_group(&antigen_wgpu::wgpu::BindGroupDescriptor {
            label: Some(format!("{}_texture_bind_group", label).as_str()),
            layout: &self.texture_bind_group_layout,
            entries: &[antigen_wgpu::wgpu::BindGroupEntry {
                binding: 0,
                resource: antigen_wgpu::wgpu::BindingResource::TextureView(
                    &texture.create_view(&antigen_wgpu::wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });

        bind_group
    }

    /// Registers a `antigen_wgpu::wgpu::Texture` with a `egui::TextureId`.
    ///
    /// This enables the application to reference
    /// the texture inside an image ui element. This effectively enables off-screen rendering inside
    /// the egui UI. Texture must have the texture format `TextureFormat::Rgba8UnormSrgb` and
    /// Texture usage `TextureUsage::SAMPLED`.
    pub fn egui_texture_from_wgpu_texture(
        &mut self,
        device: &antigen_wgpu::wgpu::Device,
        texture: &antigen_wgpu::wgpu::Texture,
    ) -> egui::TextureId {
        // We've bound it here, so that we don't add it as a pending texture.
        let bind_group = device.create_bind_group(&antigen_wgpu::wgpu::BindGroupDescriptor {
            label: Some(format!("{}_texture_bind_group", self.next_user_texture_id).as_str()),
            layout: &self.texture_bind_group_layout,
            entries: &[antigen_wgpu::wgpu::BindGroupEntry {
                binding: 0,
                resource: antigen_wgpu::wgpu::BindingResource::TextureView(
                    &texture.create_view(&antigen_wgpu::wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });
        let texture_id = egui::TextureId::User(self.next_user_texture_id);
        self.user_textures.push(Some(bind_group));
        self.next_user_texture_id += 1;

        texture_id
    }

    /// Uploads the uniform, vertex and index data used by the render pass. Should be called before `execute()`.
    pub fn update_buffers(
        &mut self,
        device: &antigen_wgpu::wgpu::Device,
        queue: &antigen_wgpu::wgpu::Queue,
        paint_jobs: &[egui::paint::ClippedMesh],
        screen_descriptor: &ScreenDescriptor,
    ) {
        let index_size = self.index_buffers.len();
        let vertex_size = self.vertex_buffers.len();

        let (logical_width, logical_height) = screen_descriptor.logical_size();

        self.update_buffer(
            device,
            queue,
            BufferType::Uniform,
            0,
            bytemuck::cast_slice(&[UniformBuffer {
                screen_size: [logical_width as f32, logical_height as f32],
            }]),
        );

        for (i, egui::ClippedMesh(_, mesh)) in paint_jobs.iter().enumerate() {
            let data: &[u8] = bytemuck::cast_slice(&mesh.indices);
            if i < index_size {
                self.update_buffer(device, queue, BufferType::Index, i, data)
            } else {
                let buffer =
                    device.create_buffer_init(&antigen_wgpu::wgpu::util::BufferInitDescriptor {
                        label: Some("egui_index_buffer"),
                        contents: data,
                        usage: antigen_wgpu::wgpu::BufferUsage::INDEX
                            | antigen_wgpu::wgpu::BufferUsage::COPY_DST,
                    });
                self.index_buffers.push(SizedBuffer {
                    buffer,
                    size: data.len(),
                });
            }

            let data: &[u8] = as_byte_slice(&mesh.vertices);
            if i < vertex_size {
                self.update_buffer(device, queue, BufferType::Vertex, i, data)
            } else {
                let buffer =
                    device.create_buffer_init(&antigen_wgpu::wgpu::util::BufferInitDescriptor {
                        label: Some("egui_vertex_buffer"),
                        contents: data,
                        usage: antigen_wgpu::wgpu::BufferUsage::VERTEX
                            | antigen_wgpu::wgpu::BufferUsage::COPY_DST,
                    });

                self.vertex_buffers.push(SizedBuffer {
                    buffer,
                    size: data.len(),
                });
            }
        }
    }

    /// Updates the buffers used by egui. Will properly re-size the buffers if needed.
    fn update_buffer(
        &mut self,
        device: &antigen_wgpu::wgpu::Device,
        queue: &antigen_wgpu::wgpu::Queue,
        buffer_type: BufferType,
        index: usize,
        data: &[u8],
    ) {
        let (buffer, storage, name) = match buffer_type {
            BufferType::Index => (
                &mut self.index_buffers[index],
                antigen_wgpu::wgpu::BufferUsage::INDEX,
                "index",
            ),
            BufferType::Vertex => (
                &mut self.vertex_buffers[index],
                antigen_wgpu::wgpu::BufferUsage::VERTEX,
                "vertex",
            ),
            BufferType::Uniform => (
                &mut self.uniform_buffer,
                antigen_wgpu::wgpu::BufferUsage::UNIFORM,
                "uniform",
            ),
        };

        if data.len() > buffer.size {
            buffer.size = data.len();
            buffer.buffer =
                device.create_buffer_init(&antigen_wgpu::wgpu::util::BufferInitDescriptor {
                    label: Some(format!("egui_{}_buffer", name).as_str()),
                    contents: bytemuck::cast_slice(data),
                    usage: storage | antigen_wgpu::wgpu::BufferUsage::COPY_DST,
                });
        } else {
            queue.write_buffer(&buffer.buffer, 0, data);
        }
    }
}

/*
impl epi::TextureAllocator for RenderPass {
    fn alloc_srgba_premultiplied(
        &mut self,
        size: (usize, usize),
        srgba_pixels: &[egui::Color32],
    ) -> egui::TextureId {
        let id = self.next_user_texture_id;
        self.next_user_texture_id += 1;

        let mut pixels = vec![0u8; srgba_pixels.len() * 4];
        for (target, given) in pixels.chunks_exact_mut(4).zip(srgba_pixels.iter()) {
            target.copy_from_slice(&given.to_array());
        }

        let (width, height) = size;
        self.pending_user_textures.push((
            id,
            egui::Texture {
                version: 0,
                width,
                height,
                pixels,
            },
        ));

        egui::TextureId::User(id)
    }

    fn free(&mut self, id: egui::TextureId) {
        if let egui::TextureId::User(id) = id {
            self.user_textures
                .get_mut(id as usize)
                .and_then(|option| option.take());
        }
    }
}
*/

// Needed since we can't use bytemuck for external types.
fn as_byte_slice<T>(slice: &[T]) -> &[u8] {
    let len = slice.len() * std::mem::size_of::<T>();
    let ptr = slice.as_ptr() as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, len) }
}
