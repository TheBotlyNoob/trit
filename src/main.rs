mod app;

async fn main_() {
    if let Err(e) = app::main().await {
        tracing::error!("Error: {}", e);
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        tracing_wasm::set_as_global_default();

        wasm_bindgen_futures::spawn_local(main_());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing_subscriber::fmt::init();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(main_());
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
