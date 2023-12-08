use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    time::SystemTime,
};

use antigen_wgpu::{wgpu::TextureFormat, Render, WgpuDevice, WgpuQueue};
use antigen_winit::{winit::event::ModifiersState, WinitWindowEvent};
use async_std::sync::Arc;
use clipboard::ClipboardContext;
use deebs::{
    macros::{CommonKeys, Row},
    BorrowColumn, CommonKeys, ReadCell, Row, Table,
};
use egui::{CtxRef, Output, Pos2, RawInput, Rect};
use futures::StreamExt;

use crate::{EguiRenderPass, ScreenDescriptor};

#[cfg(feature = "clipboard")]
use clipboard::ClipboardProvider;

pub trait UserInterface {
    fn user_interface(&mut self, context: &CtxRef);
}

impl<F> UserInterface for F
where
    F: Fn(&CtxRef),
{
    fn user_interface(&mut self, context: &CtxRef) {
        (self)(context)
    }
}

/// Helper typedef for using boxed dyn instead of generics for EguiUserInterface polymorphism.
pub type BoxedDyn = Box<dyn UserInterface + Send + Sync>;

impl UserInterface for BoxedDyn {
    fn user_interface(&mut self, context: &CtxRef) {
        self.deref_mut().user_interface(context)
    }
}

/// An egui-backed UI-as-a-value.
pub struct EguiUserInterface<T, F = BoxedDyn>
where
    F: UserInterface + Send + Sync,
{
    pub table: Arc<T>,
    pub context: CtxRef,
    pub screen_rect: Rect,
    pub scale_factor: f32,
    pub raw_input: RawInput,
    pub pointer_position: Pos2,
    pub pointer_exclusive: bool,
    pub modifiers: ModifiersState,
    pub output: Output,
    pub render_pass: EguiRenderPass,
    pub ui: F,
    #[cfg(feature = "clipboard")]
    pub clipboard: ClipboardContext,
}

impl<T, F> EguiUserInterface<T, F>
where
    T: Table + BorrowColumn<WgpuDevice> + Send + Sync,
    F: UserInterface + Send + Sync + 'static,
{
    pub async fn new(table: Arc<T>, output_format: TextureFormat, ui: F) -> Self
where {
        let render_pass = {
            let device = table
                .get::<WgpuDevice>(
                    &table
                        .keys::<WgpuDevice>()
                        .await
                        .next()
                        .await
                        .expect("No WgpuDevice cell in table."),
                )
                .await
                .unwrap();

            EguiRenderPass::new(device.deref(), output_format).await
        };

        EguiUserInterface {
            table,
            render_pass,
            context: Default::default(),
            screen_rect: Rect::NOTHING,
            scale_factor: 1.0,
            pointer_position: Default::default(),
            pointer_exclusive: Default::default(),
            modifiers: Default::default(),
            raw_input: Default::default(),
            output: Default::default(),
            ui,
            #[cfg(feature = "clipboard")]
            clipboard: ClipboardContext::new().expect("Failed to get clipboard context."),
        }
    }
}

impl<T> EguiUserInterface<T, BoxedDyn>
where
    T: Table + BorrowColumn<WgpuDevice> + Send + Sync,
{
    pub async fn boxed<F>(table: Arc<T>, output_format: TextureFormat, ui: F) -> Self
    where
        F: UserInterface + Send + Sync + 'static,
    {
        Self::new(table, output_format, Box::new(ui)).await
    }
}

impl<T, F> Debug for EguiUserInterface<T, F>
where
    F: UserInterface + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EguiUserInterface")
    }
}

impl<T, F> Display for EguiUserInterface<T, F>
where
    F: UserInterface + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EguiUserInterface")
    }
}

impl<T, F> egui::Widget for &EguiUserInterface<T, F>
where
    F: UserInterface + Send + Sync,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(self.to_string())
    }
}

impl<T, F> Render for EguiUserInterface<T, F>
where
    T: Table + BorrowColumn<WgpuDevice> + BorrowColumn<WgpuQueue> + Send + Sync,
    F: UserInterface + Send + Sync,
{
    fn render(
        &mut self,
        view: &antigen_wgpu::wgpu::TextureView,
    ) -> antigen_wgpu::wgpu::CommandBuffer {
        async_std::task::block_on(async move {
            let DeviceQueueRow { device, queue } = DeviceQueueRow::new(
                self.table.deref(),
                &DeviceQueueRow::common_keys(self.table.deref())
                    .await
                    .next()
                    .await
                    .expect("No DeviceQueueRow in table."),
            )
            .await;

            let screen_rect = self.screen_rect;
            let scale_factor = self.scale_factor;

            {
                self.raw_input.screen_rect = Some(screen_rect);
                self.raw_input.pixels_per_point = Some(scale_factor);
                self.raw_input.time = Some(
                    SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                );

                let input = self.raw_input.take();
                self.context.begin_frame(input);
            }

            self.ui.user_interface(&self.context);

            // End the UI frame. We could now handle the output and draw the UI with the backend.
            let (output, paint_commands) = self.context.end_frame();

            #[cfg(feature = "clipboard")]
            handle_clipboard(&output, &mut self.clipboard);

            #[cfg(feature = "webbrowser")]
            handle_links(&output);

            self.output = output;

            let paint_jobs = self.context.tessellate(paint_commands);
            let egui_texture = self.context.texture();
            self.render_pass
                .update_texture(&device, &queue, egui_texture.deref());

            //render_pass.update_user_textures(&device, &queue);

            // Upload all resources for the GPU.
            let screen_descriptor = ScreenDescriptor {
                physical_width: screen_rect.width() as u32,
                physical_height: screen_rect.height() as u32,
                scale_factor,
            };

            self.render_pass
                .update_buffers(&device, &queue, &paint_jobs[..], &screen_descriptor);

            // Record all render passes.
            let mut encoder =
                device.create_command_encoder(&antigen_wgpu::wgpu::CommandEncoderDescriptor {
                    label: Some("egui_command_encoder"),
                });

            self.render_pass.execute(
                &mut encoder,
                view,
                &paint_jobs[..],
                &screen_descriptor,
                Some(antigen_wgpu::wgpu::Color::BLUE),
            );

            encoder.finish()
        })
    }
}

#[derive(Row, CommonKeys)]
struct DeviceQueueRow<'a> {
    device: ReadCell<'a, WgpuDevice>,
    queue: ReadCell<'a, WgpuQueue>,
}

/// Returns `true` if egui should handle the event exclusively. Check this to
/// avoid unexpected interactions, e.g. a mouse click registering "behind" the UI.
pub fn captures_event(context: &CtxRef, event: WinitWindowEvent) -> bool {
    match event {
        WinitWindowEvent::ReceivedCharacter(_)
        | WinitWindowEvent::KeyboardInput { .. }
        | WinitWindowEvent::ModifiersChanged(_) => context.wants_keyboard_input(),
        WinitWindowEvent::MouseWheel { .. } | WinitWindowEvent::MouseInput { .. } => {
            context.wants_pointer_input()
        }
        WinitWindowEvent::CursorMoved { .. } => context.is_using_pointer(),
        _ => false,
    }
}

#[cfg(feature = "webbrowser")]
fn handle_links(output: &egui::Output) {
    if let Some(url) = &output.open_url {
        if let Err(err) = webbrowser::open(&url.url) {
            eprintln!("Failed to open url: {}", err);
        }
    }
}

#[cfg(feature = "clipboard")]
fn handle_clipboard(output: &egui::Output, clipboard: &mut ClipboardContext) {
    if !output.copied_text.is_empty() {
        if let Err(err) = clipboard.set_contents(output.copied_text.clone()) {
            eprintln!("Copy/Cut error: {}", err);
        }
    }
}
