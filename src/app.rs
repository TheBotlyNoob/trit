use softbuffer::GraphicsContext;
use tiny_skia::{ClipMask, Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Shader, Transform};
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

                let paint1 = Paint {
                    anti_alias: true,
                    shader: Shader::SolidColor(Color::from_rgba8(50, 127, 150, 200)),
                    ..Default::default()
                };

                // makes a rectangle with rounded corners
                let path1 = {
                    let (x, y) = (100.0, 100.0);
                    let (w, h) = (200.0, 200.0);
                    let border_radius = 50.0;

                    let mut pb = PathBuilder::new();
                    pb.move_to(x, y);
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
                };

                let mut pixmap = Pixmap::new(width, height).unwrap();
                pixmap.fill_path(
                    &path1,
                    &paint1,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
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
