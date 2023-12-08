use antigen_winit_wgpu::WinitSwapChain;
use futures::StreamExt;
use std::{
    borrow::{Borrow, Cow},
    ops::Deref,
    sync::atomic::AtomicUsize,
};

use antigen_rendering::RedrawFlag;
use antigen_wgpu::{
    wgpu::{
        Color, CommandBuffer, FragmentState, LoadOp, MultisampleState, Operations,
        PipelineLayoutDescriptor, PresentMode, PrimitiveState, RenderPassColorAttachment,
        RenderPassDescriptor, RenderPipelineDescriptor, ShaderFlags, ShaderModuleDescriptor,
        ShaderSource, TextureFormat, TextureUsage, TextureView, VertexState,
    },
    WgpuCommandBuffers, WgpuDevice, WgpuRenderer, WgpuSwapChainFrame,
};
use antigen_winit::{WindowDescriptor, WinitWindow};
use async_std::sync::Arc;
use deebs::{
    macros::{CommonKeys, Map, Row},
    BorrowColumn, Insert, ReadCell, Table,
};

use antigen_components::Label;

/// A user-created row query result holding references to all cells in a table.
#[derive(Debug, Row, CommonKeys, Map)]
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
        + BorrowColumn<WinitWindow>
        + BorrowColumn<WinitSwapChain>
        + BorrowColumn<WgpuSwapChainFrame>
        + BorrowColumn<WgpuRenderer>
        + BorrowColumn<WgpuCommandBuffers>
        + BorrowColumn<RedrawFlag>
        + Send
        + Sync
        + 'static,
{
    tracing::info!("Assembling hello_triangle...");

    let window_key = table.next_key();

    table
        .insert(window_key, Label::from("Hello Triangle"))
        .await;

    antigen_winit_wgpu::InsertRow::insert(
        table.deref(),
        window_key,
        (
            WinitWindow::Pending(WindowDescriptor {
                title: Some("Hello Triangle".into()),
                visible: Some(false),
                ..Default::default()
            }),
            RedrawFlag(true),
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
}

pub async fn renderer<T>(
    table: Arc<T>,
    texture_format: TextureFormat,
) -> impl Fn(&TextureView) -> CommandBuffer + Send + Sync
where
    T: Table + BorrowColumn<WgpuDevice> + Send + Sync,
{
    let render_pipeline = {
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
            label: Some("hello_triangle_shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            flags: ShaderFlags::all(),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("hello_triangle_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("hello_triangle_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[texture_format.into()],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
        })
    };

    move |view| {
        async_std::task::block_on(async {
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

            let mut encoder =
                device.create_command_encoder(&antigen_wgpu::wgpu::CommandEncoderDescriptor {
                    label: Some("hello_triangle_command_encoder"),
                });

            {
                let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("hello_triangle_render_pass"),
                    color_attachments: &[RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::GREEN),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
                rpass.set_pipeline(&render_pipeline);
                rpass.draw(0..3, 0..1);
            }

            encoder.finish()
        })
    }
}
