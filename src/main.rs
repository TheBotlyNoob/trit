#![warn(clippy::nursery, clippy::pedantic)]

use softbuffer::GraphicsContext;
use winit::{event_loop::EventLoop, window::WindowBuilder};

mod app;
mod dom;

// bootstrap for both native and wasm - the main code is in `app.rs`

fn main_() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Trit").build(&event_loop)?;

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

    // SAFETY: both the window and the graphics context live for the life of main
    let gfx_ctx = unsafe { GraphicsContext::new(&window, &window) }?;

    app::event_loop(window, event_loop, gfx_ctx)
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        tracing_wasm::set_as_global_default();
    }
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    if let Err(e) = main_() {
        tracing::error!(?e);
    }
}
#[cfg(target_arch = "wasm32")]
mod _wasm_allocator {
    use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};

    // SAFETY: This application is single threaded, so using AssumeSingleThreaded is allowed.
    #[global_allocator]
    static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
        unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };
}
