use std::{
    borrow::{Borrow, Cow},
    fmt::Display,
    ops::Deref,
    sync::atomic::AtomicUsize,
};

use antigen_components::Label;
use antigen_rendering::{AlwaysRedraw, OnCpu, RedrawFlag};
use antigen_wgpu::{
    wgpu::{
        util::DeviceExt, vertex_attr_array, Color, CommandBuffer, FragmentState, LoadOp,
        MultisampleState, Operations, PipelineLayoutDescriptor, PresentMode, PrimitiveState,
        PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor,
        RenderPipelineDescriptor, ShaderFlags, ShaderModuleDescriptor, ShaderSource, TextureFormat,
        TextureUsage, TextureView, VertexBufferLayout, VertexState,
    },
    WgpuCommandBuffers, WgpuDevice, WgpuQueue, WgpuRenderer, WgpuSwapChainFrame,
};
use antigen_winit::{WindowDescriptor, WinitWindow};
use antigen_winit_wgpu::WinitSwapChain;
use deebs::{
    array_stream,
    macros::{CommonKeys, Insert, Row},
    BorrowColumn, CommonKeys, Insert, ReadCell, Row, Table,
};

use async_std::sync::Arc;
use futures::StreamExt;

const QUAD_COUNT: usize = 16;

#[derive(Debug, Default, Copy, Clone)]
pub struct QuadPosition {
    pub x: f32,
    pub y: f32,
}

impl egui::Widget for &mut QuadPosition {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label("X:")
                .union(
                    ui.add(
                        egui::widgets::DragValue::new(&mut self.x)
                            .fixed_decimals(3)
                            .speed(0.001),
                    ),
                )
                .union(ui.label("Y:"))
                .union(
                    ui.add(
                        egui::widgets::DragValue::new(&mut self.y)
                            .fixed_decimals(3)
                            .speed(0.001),
                    ),
                )
        })
        .response
    }
}

impl IntoIterator for QuadPosition {
    type Item = f32;

    type IntoIter = std::array::IntoIter<f32, 2>;

    fn into_iter(self) -> Self::IntoIter {
        std::array::IntoIter::new([self.x, self.y])
    }
}

impl Display for QuadPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}, {}", self.x, self.y))
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct QuadSize {
    pub w: f32,
    pub h: f32,
}

impl egui::Widget for &mut QuadSize {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label("W:")
                .union(
                    ui.add(
                        egui::widgets::DragValue::new(&mut self.w)
                            .fixed_decimals(3)
                            .speed(0.001),
                    ),
                )
                .union(ui.label("H:"))
                .union(
                    ui.add(
                        egui::widgets::DragValue::new(&mut self.h)
                            .fixed_decimals(3)
                            .speed(0.001),
                    ),
                )
        })
        .response
    }
}

impl IntoIterator for QuadSize {
    type Item = f32;

    type IntoIter = std::array::IntoIter<f32, 2>;

    fn into_iter(self) -> Self::IntoIter {
        std::array::IntoIter::new([self.w, self.h])
    }
}

impl Display for QuadSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}, {}", self.w, self.h))
    }
}

#[derive(Row, Insert, CommonKeys)]
struct QuadRow<'a> {
    position: ReadCell<'a, QuadPosition>,
    size: ReadCell<'a, QuadSize>,
}

/// A user-created row query result holding references to all cells in a table.
#[derive(Debug, Row)]
pub struct DebugRow<'a> {
    pub label: Option<ReadCell<'a, Label>>,

    pub wgpu: Option<ReadCell<'a, WgpuDevice>>,

    pub window: Option<ReadCell<'a, WinitWindow>>,
    pub swap_chain: Option<ReadCell<'a, WinitSwapChain>>,
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn assemble<T>(table: Arc<T>)
where
    T: Table
        + Borrow<AtomicUsize>
        + BorrowColumn<Label>
        + BorrowColumn<WgpuDevice>
        + BorrowColumn<WgpuQueue>
        + BorrowColumn<WinitWindow>
        + BorrowColumn<WinitSwapChain>
        + BorrowColumn<RedrawFlag>
        + BorrowColumn<AlwaysRedraw<OnCpu>>
        + BorrowColumn<WgpuSwapChainFrame>
        + BorrowColumn<WgpuRenderer>
        + BorrowColumn<WgpuCommandBuffers>
        + BorrowColumn<QuadPosition>
        + BorrowColumn<QuadSize>
        + Send
        + Sync
        + 'static,
{
    tracing::info!("Assembling hello_quads...");

    // Create window
    let window_key = table.next_key();

    table.insert(window_key, Label::from("Hello Quads")).await;

    antigen_winit_wgpu::InsertRow::insert(
        table.deref(),
        window_key,
        (
            WinitWindow::Pending(WindowDescriptor {
                title: Some("Hello Quads".into()),
                visible: Some(false),
                ..Default::default()
            }),
            RedrawFlag::default(),
            WinitSwapChain::Pending(antigen_wgpu::wgpu::SwapChainDescriptor {
                usage: TextureUsage::RENDER_ATTACHMENT,
                format: TextureFormat::Bgra8Unorm,
                width: 800,
                height: 600,
                present_mode: PresentMode::Mailbox,
            }),
            WgpuSwapChainFrame::default(),
            WgpuCommandBuffers::default(),
        ),
    )
    .await;

    table
        .insert(
            window_key,
            WgpuRenderer::boxed(renderer(table.clone(), TextureFormat::Bgra8Unorm).await),
        )
        .await;

    table
        .insert(window_key, AlwaysRedraw::<OnCpu>::default())
        .await;

    // Create quad entities
    QuadRow::insert_auto_multi(
        table.deref(),
        std::array::IntoIter::new([
            (
                QuadPosition { x: -0.5, y: -0.5 },
                QuadSize { w: 0.25, h: 0.25 },
            ),
            (
                QuadPosition { x: 0.5, y: 0.5 },
                QuadSize { w: 0.25, h: 0.25 },
            ),
            (
                QuadPosition { x: 0.4, y: -0.4 },
                QuadSize { w: 0.5, h: 0.5 },
            ),
            (
                QuadPosition { x: -0.4, y: 0.4 },
                QuadSize { w: 0.5, h: 0.5 },
            ),
            (QuadPosition { x: 0.0, y: 0.0 }, QuadSize { w: 0.1, h: 0.1 }),
        ]),
    )
    .await;
}

pub async fn renderer<T>(
    table: Arc<T>,
    texture_format: TextureFormat,
) -> impl Fn(&TextureView) -> CommandBuffer + Send + Sync
where
    T: Table
        + BorrowColumn<WgpuDevice>
        + BorrowColumn<WgpuQueue>
        + BorrowColumn<QuadPosition>
        + BorrowColumn<QuadSize>
        + Send
        + Sync,
{
    let (render_pipeline, vertex_buffer) = {
        let device = table
            .get::<WgpuDevice>(
                &table
                    .keys::<WgpuDevice>()
                    .await
                    .next()
                    .await
                    .expect("No WgpuDevice in table."),
            )
            .await
            .unwrap();

        let shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("hello_quads_shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            flags: ShaderFlags::all(),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("hello_quads_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("hello_quads_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: 4 * 4,
                    step_mode: antigen_wgpu::wgpu::InputStepMode::Instance,
                    attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[texture_format.into()],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
        });

        let vertex_buffer_data = [0.0; QUAD_COUNT * 4];
        let vertex_buffer =
            device.create_buffer_init(&antigen_wgpu::wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::bytes_of(&vertex_buffer_data),
                usage: antigen_wgpu::wgpu::BufferUsage::VERTEX
                    | antigen_wgpu::wgpu::BufferUsage::COPY_DST,
            });

        (render_pipeline, vertex_buffer)
    };

    move |view| {
        async_std::task::block_on(async {
            #[derive(Row, CommonKeys)]
            struct DeviceQueueRow<'a> {
                device: ReadCell<'a, WgpuDevice>,
                queue: ReadCell<'a, WgpuQueue>,
            }

            let DeviceQueueRow { device, queue } = DeviceQueueRow::new(
                table.deref(),
                &DeviceQueueRow::common_keys(table.deref())
                    .await
                    .next()
                    .await
                    .expect("No DeviceQueueRow in table."),
            )
            .await;

            // Fetch quad information from table
            let mut quads = vec![];
            let mut stream = QuadRow::common_keys(table.deref()).await;
            while let Some(key) = stream.next().await {
                let QuadRow { position, size } = QuadRow::new(table.deref(), &key).await;
                if quads.len() < QUAD_COUNT {
                    quads.push((*position.deref(), *size.deref()));
                }
            }

            let mut floats = quads
                .into_iter()
                .flat_map(|(position, size)| position.into_iter().chain(size.into_iter()))
                .collect::<Vec<f32>>();

            // Upload vertex data
            let instance_count = (QUAD_COUNT).min(floats.len() / 4);
            let float_count = (QUAD_COUNT * 4).min(floats.len());

            let mut buf: [f32; QUAD_COUNT * 4] = [0.0; QUAD_COUNT * 4];
            for (i, v) in floats.drain(0..float_count).enumerate() {
                buf[i] = v;
            }

            queue.write_buffer(&vertex_buffer, 0, bytemuck::bytes_of(&buf));

            // Run render pass
            let mut encoder =
                device.create_command_encoder(&antigen_wgpu::wgpu::CommandEncoderDescriptor {
                    label: Some("hello_quads_command_encoder"),
                });

            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("hello_quads_render_pass"),
                    color_attachments: &[RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&render_pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..4, 0..instance_count as u32);
            }

            encoder.finish()
        })
    }
}
