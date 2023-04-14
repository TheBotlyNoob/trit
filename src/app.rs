use softbuffer::GraphicsContext;
use tiny_skia::{
    FillRule, FilterQuality, Paint, PathBuilder, Pattern, Pixmap, SpreadMode, Transform,
};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        window.set_inner_size(winit::dpi::PhysicalSize {
            width: web_sys::window()
                .unwrap()
                .inner_width()
                .unwrap()
                .as_f64()
                .unwrap(),
            height: web_sys::window()
                .unwrap()
                .inner_height()
                .unwrap()
                .as_f64()
                .unwrap(),
        });

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&window.canvas())
            .unwrap();
    }

    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };

                let triangle = crate_triangle();

                let paint = Paint {
                    anti_alias: true,
                    shader: Pattern::new(
                        triangle.as_ref(),
                        SpreadMode::Repeat,
                        FilterQuality::Bicubic,
                        1.0,
                        Transform::from_row(1.5, -0.4, 0.0, -0.8, 5.0, 1.0),
                    ),
                    ..Default::default()
                };

                let path = PathBuilder::from_circle(200.0, 200.0, 180.0).unwrap();

                let mut pixmap = Pixmap::new(width, height).unwrap();
                pixmap.fill_path(
                    &path,
                    &paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );

                fn crate_triangle() -> Pixmap {
                    let mut paint = Paint::default();
                    paint.set_color_rgba8(50, 127, 150, 200);
                    paint.anti_alias = true;

                    let mut pb = PathBuilder::new();
                    pb.move_to(0.0, 20.0);
                    pb.line_to(20.0, 20.0);
                    pb.line_to(10.0, 0.0);
                    pb.close();
                    let path = pb.finish().unwrap();

                    let mut pixmap = Pixmap::new(20, 20).unwrap();
                    pixmap.fill_path(
                        &path,
                        &paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                    pixmap
                }

                graphics_context.set_buffer(
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
