use softbuffer::GraphicsContext;
use tiny_skia::{ClipMask, FillRule, Paint, PathBuilder, Pixmap, Rect, Transform};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

pub async fn event_loop(
    window: Window,
    event_loop: EventLoop<()>,
    mut gfx_ctx: GraphicsContext,
) -> Result<(), Box<dyn std::error::Error>> {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let PhysicalSize { width, height } = window.inner_size();
                let mut pixmap = Pixmap::new(width, height).unwrap();

                let clip_path = {
                    let mut pb = PathBuilder::new();
                    pb.push_circle(250.0, 250.0, 200.0);
                    pb.push_circle(250.0, 250.0, 100.0);
                    pb.finish().unwrap()
                };

                let clip_path = clip_path
                    .transform(Transform::from_row(1.0, -0.3, 0.0, 1.0, 0.0, 75.0))
                    .unwrap();

                let mut clip_mask = ClipMask::new();
                clip_mask.set_path(500, 500, &clip_path, FillRule::EvenOdd, true);

                let mut paint = Paint::default();
                paint.set_color_rgba8(50, 127, 150, 200);
                pixmap.fill_rect(
                    Rect::from_xywh(0.0, 0.0, 500.0, 500.0).unwrap(),
                    &paint,
                    Transform::identity(),
                    Some(&clip_mask),
                );

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
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
