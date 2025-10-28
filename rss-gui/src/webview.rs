use std::thread;

#[allow(dead_code)]
pub fn open_webview(url: &str, title: &str) -> Result<(), String> {
    let url = url.to_string();
    let title = title.to_string();
    thread::Builder::new()
        .name("readrss-webview".to_string())
        .spawn(move || {
            use wry::application::event::{Event, WindowEvent};
            use wry::application::event_loop::{ControlFlow, EventLoop};
            use wry::application::window::WindowBuilder;
            use wry::webview::WebViewBuilder;
            use wry::dpi::LogicalSize;

            let event_loop = EventLoop::new();
            let window = WindowBuilder::new()
                .with_title(title)
                .with_inner_size(LogicalSize::new(800.0, 800.0))
                .build(&event_loop)
                .map_err(|e| e.to_string())?;

            let _webview = WebViewBuilder::new()
                .map_err(|e| e.to_string())?
                .with_url(&url)
                .map_err(|e| e.to_string())?
                .with_devtools(false)
                .build(&window)
                .map_err(|e| e.to_string())?;

            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Wait;
                match event {
                    Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            });
        })
        .map(|_| ())
        .map_err(|e| e.to_string())
}
