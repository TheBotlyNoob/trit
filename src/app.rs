use softbuffer::GraphicsContext;
use tiny_skia::{Path, PathBuilder, Pixmap, Rect};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::Window;

#[allow(clippy::unused_async, clippy::future_not_send)]
pub async fn event_loop(
    window: Window,
    event_loop: EventLoop<()>,
    mut gfx_ctx: GraphicsContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let dom = crate::dom::ArenaDom::default();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let PhysicalSize { width, height } = window.inner_size();

                let pixmap = Pixmap::new(width, height).unwrap();

                #[allow(clippy::cast_possible_truncation)]
                gfx_ctx.set_buffer(
                    bytemuck::cast_slice(pixmap.pixels()),
                    width as u16,
                    height as u16,
                );
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                control_flow.set_exit();
            }
            _ => {}
        }
    });
}

fn rounded_rect(rect: Rect, border_radius: f32) -> Path {
    let (x, y) = (rect.x(), rect.y());
    let (w, h) = (rect.width(), rect.height());

    let mut pb = PathBuilder::new();
    pb.move_to(x + border_radius, y);
    pb.line_to(x + w - border_radius, y);
    pb.quad_to(x + w, y, x + w, y + border_radius);
    pb.line_to(x + w, y + h - border_radius);
    pb.quad_to(x + w, y + h, x + w - border_radius, y + h);
    pb.line_to(x + border_radius, y + h);
    pb.quad_to(x, y + h, x, y + h - border_radius);
    pb.line_to(x, y + border_radius);
    pb.quad_to(x, y, x + border_radius, y);
    pb.close();
    pb.finish().unwrap()
}
