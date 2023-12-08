use async_std::sync::Arc;
use deebs::{macros::{Row, CommonKeys}, BorrowColumn, CommonKeys, ReadCell, Row, Table, WriteCell};
use futures::StreamExt;

use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use wgpu::{CommandBuffer, TextureView};

use crate::{WgpuCommandBuffers, WgpuTextureView};

pub trait Render {
    fn render(&mut self, view: &TextureView) -> CommandBuffer;
}

impl<T> Render for T
where
    T: Fn(&TextureView) -> CommandBuffer,
{
    fn render(&mut self, view: &TextureView) -> CommandBuffer {
        (self)(view)
    }
}

pub type BoxedDyn = Box<dyn Render + Send + Sync>;

impl Render for BoxedDyn {
    fn render(&mut self, view: &TextureView) -> CommandBuffer {
        self.deref_mut().render(view)
    }
}

#[cfg_attr(feature = "egui", derive(deebs::macros::Widget))]
pub struct WgpuRenderer<F = BoxedDyn>
where
    F: Render + Send + Sync,
{
    render: F,
}

impl<F> Render for WgpuRenderer<F>
where
    F: Render + Send + Sync,
{
    fn render(&mut self, view: &TextureView) -> CommandBuffer {
        self.render.render(view)
    }
}

impl<F> WgpuRenderer<F>
where
    F: Render + Send + Sync,
{
    pub fn new(render: F) -> Self {
        WgpuRenderer { render }
    }

    pub fn render(&mut self, view: &TextureView) -> CommandBuffer {
        self.render.render(view)
    }
}

impl WgpuRenderer<BoxedDyn> {
    pub fn boxed<F>(render: F) -> Self
    where
        F: Render + Send + Sync + 'static,
    {
        Self::new(Box::new(render))
    }
}

impl<F> Debug for WgpuRenderer<F>
where
    F: Render + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WgpuRenderer")
    }
}

impl<F> Display for WgpuRenderer<F>
where
    F: Render + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WgpuRenderer")
    }
}

pub async fn run_texture_view_render_system<F, T>(table: Arc<T>)
where
    T: Table
        + BorrowColumn<WgpuTextureView>
        + BorrowColumn<F>
        + BorrowColumn<WgpuCommandBuffers>
        + Send
        + Sync,
    F: Render + Send + Sync + 'static,
{
    #[derive(Row, CommonKeys)]
    struct RenderRow<'a, F>
    where
        F: Render + Send + Sync + 'static,
    {
        texture_view: ReadCell<'a, WgpuTextureView>,
        renderer: WriteCell<'a, F>,
        command_buffers: WriteCell<'a, WgpuCommandBuffers>,
    }

    let mut stream = RenderRow::<F>::common_keys(table.deref()).await;
    while let Some(key) = stream.next().await {
        let RenderRow {
            texture_view,
            mut renderer,
            mut command_buffers,
        } = RenderRow::<F>::new(table.deref(), &key).await;

        if let WgpuTextureView::Ready(texture_view) = texture_view.deref() {
            command_buffers.push(renderer.render(texture_view));
        }
    }
}
