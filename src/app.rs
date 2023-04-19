use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, SwashCache};
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeSink;
use html5ever::{parse_document, ParseOpts};
use html_tags::ElementOwned;
use softbuffer::GraphicsContext;
use tiny_skia::{Paint, Path, PathBuilder, Pixmap, Rect, Transform};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::dom::{Dom, Node, NodeHandle};

#[allow(clippy::too_many_lines)]
pub fn event_loop(
    window: Window,
    event_loop: EventLoop<()>,
    mut gfx_ctx: GraphicsContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut font_system = FontSystem::new();
    let mut swash_cache = SwashCache::new();
    let default_metrics = Metrics::new(14.0, 20.0);

    let mut buffer = Buffer::new(&mut font_system, default_metrics);

    let mut dom = {
        let dom = parse_document(Dom::default(), ParseOpts::default());

        dom.one(include_str!("../test.html"))
    };

    let mut url = String::with_capacity(16);

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let PhysicalSize { width, height } = window.inner_size();

                #[allow(clippy::cast_precision_loss)]
                buffer.set_size(&mut font_system, width as _, height as _);

                let mut pixmap = Pixmap::new(width, height).unwrap();
                pixmap.fill(tiny_skia::Color::WHITE);

                let root = dom.get_document();

                let root_nodes = dom
                    .map()
                    .values()
                    .filter(|n| matches!(n, Node::Element { parent, .. } if *parent == root));

                // TODO: external stylesheets
                let style_nodes = dom
                    .map()
                    .values()
                    .filter(|n| matches!(n, Node::Element { elem, .. } if matches!(elem, ElementOwned::Style(_))));
                
                for node in root_nodes {
                    let Node::Element { elem, children, .. } = node else {
                        continue; // this can't happen 
                    };

                    if !matches!(elem, ElementOwned::Script(_) | ElementOwned::Style(_)) {
                      render_text(&dom, elem, children, &mut pixmap, &mut buffer, &mut font_system, &mut swash_cache);
                    }
                }

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
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            // user input
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let Some(keycode) = input.virtual_keycode {
                    if input.state == winit::event::ElementState::Pressed {
                        match keycode {
                            winit::event::VirtualKeyCode::Escape => {
                                control_flow.set_exit();
                            }
                            winit::event::VirtualKeyCode::Return => {
                                tracing::warn!("TODO: navigate to {url}");
                            }
                            winit::event::VirtualKeyCode::Back => {
                                url.pop();
                            }

                            _ => {}
                        }
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(c),
                ..
            } => {
                url.push(c);
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

fn render_text(dom: &Dom, elem: &ElementOwned, children: &Vec<NodeHandle>, pixmap: &mut Pixmap, buffer: &mut Buffer, font_system: &mut FontSystem, swash_cache: &mut SwashCache) {
    let mut text = String::new();
    for &child in children {
        let node = dom.map().get(child);
        if let Some(Node::Text { contents }) = node {
            text.push_str(if matches!(elem, ElementOwned::Pre(_)) {
                contents
            } else {
                contents.trim()
            });
        }
    }
    let attrs = Attrs::new();
    buffer.set_text(font_system, &text, attrs);
    buffer.draw(
        font_system,
        swash_cache,
        cosmic_text::Color::rgb(0, 0, 0),
        |x, y, w, h, color| {
            #[allow(clippy::cast_precision_loss)]
            let rect =
                Rect::from_xywh(x as _, y as _, w as _, h as _).unwrap();
            pixmap.fill_rect(
                rect,
                &Paint {
                    shader: tiny_skia::Shader::SolidColor(
                        tiny_skia::Color::from_rgba8(
                            color.r(),
                            color.g(),
                            color.b(),
                            color.a(),
                        ),
                    ),
                    anti_alias: true,
                    ..Default::default()
                },
                Transform::identity(),
                None,
            );
        },
    );
}
