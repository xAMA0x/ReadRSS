use std::process::Command;
use std::ffi::OsString;

// Lance une instance enfant du même binaire en mode "webview-child".
// Ceci isole la boucle d'événements sur le thread principal du processus enfant
// et empêche la fermeture de la WebView de fermer l'appli principale.
#[allow(dead_code)]
pub fn open_webview(url: &str, title: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let status = Command::new(exe)
        .arg("--webview-child")
        .arg("--webview-url").arg(url)
        .arg("--webview-title").arg(title)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err(format!("Processus webview enfant terminé avec code {:?}", status.code())) }
}

// Ouvre une WebView avec un HTML local minimal, utile pour diagnostiquer un écran blanc.
#[allow(dead_code)]
pub fn open_webview_local_html(title: &str, html: &str) -> Result<(), String> {
    let mut temp = std::env::temp_dir();
    temp.push("readrss_webview_test.html");
    std::fs::write(&temp, html).map_err(|e| e.to_string())?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let status = Command::new(exe)
        .arg("--webview-child")
        .arg("--webview-title").arg(title)
        .arg("--webview-html-file").arg(temp.as_os_str())
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err(format!("Processus webview enfant terminé avec code {:?}", status.code())) }
}

// Mode enfant: à appeler depuis main() très tôt si les flags sont présents.
// Retourne Some(code) si le mode enfant a été exécuté, sinon None.
pub fn maybe_run_webview_child_from_args() -> Option<i32> {
    let mut args = std::env::args_os();
    let mut is_child = false;
    let mut url: Option<OsString> = None;
    let mut title: Option<OsString> = None;
    let mut html_file: Option<OsString> = None;

    while let Some(arg) = args.next() {
        if arg == "--webview-child" { is_child = true; continue; }
        if arg == "--webview-url" { url = args.next(); continue; }
    if arg == "--webview-title" { title = args.next(); continue; }
    if arg == "--webview-html-file" { html_file = args.next(); continue; }
    }

    if !is_child { return None; }
    let url = url.unwrap_or_else(|| OsString::from("about:blank"));
    let title = title.unwrap_or_else(|| OsString::from("Aperçu"));

    let url = url.to_string_lossy().to_string();
    let title = title.to_string_lossy().to_string();

    let html_path = html_file.map(|s| s.to_string_lossy().to_string());
    run_webview_window(&url, &title, html_path.as_deref());
    Some(0)
}

fn run_webview_window(url: &str, title: &str, html_path: Option<&str>) {
    // Workarounds Linux/X11 pour écrans blancs avec certaines piles graphiques
    if std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none() {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
    if std::env::var_os("WEBKIT_USE_SINGLE_WEB_PROCESS").is_none() {
        std::env::set_var("WEBKIT_USE_SINGLE_WEB_PROCESS", "1");
    }
    if std::env::var_os("WEBKIT_DISABLE_GPU_PROCESS").is_none() {
        std::env::set_var("WEBKIT_DISABLE_GPU_PROCESS", "1");
    }
    if std::env::var_os("WEBKIT_DISABLE_WEBGL").is_none() {
        std::env::set_var("WEBKIT_DISABLE_WEBGL", "1");
    }
    if std::env::var_os("LIBGL_DRI3_DISABLE").is_none() {
        std::env::set_var("LIBGL_DRI3_DISABLE", "1");
    }
    if std::env::var_os("GSK_RENDERER").is_none() { std::env::set_var("GSK_RENDERER", "cairo"); }
    if std::env::var_os("GDK_GL").is_none() { std::env::set_var("GDK_GL", "software"); }

    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
    #[cfg(target_os = "linux")]
    use tao::platform::unix::EventLoopBuilderExtUnix;
    use tao::window::WindowBuilder;
    use tao::dpi::LogicalSize;
    use wry::WebViewBuilder;

    #[cfg(target_os = "linux")]
    let event_loop: EventLoop<()> = EventLoopBuilder::new().with_any_thread(false).build();
    #[cfg(not(target_os = "linux"))]
    let event_loop: EventLoop<()> = EventLoop::new();

    let window = match WindowBuilder::new()
        .with_title(title)
        .with_inner_size(LogicalSize::new(800.0, 800.0))
        .with_decorations(true)
        .build(&event_loop)
    {
        Ok(w) => w,
        Err(e) => { eprintln!("Impossible de créer la fenêtre WebView: {}", e); return; }
    };

    let builder = WebViewBuilder::new(&window);
    let builder = if let Some(path) = html_path {
        match std::fs::read_to_string(path) {
            Ok(html) => builder.with_html(html),
            Err(e) => {
                eprintln!("Impossible de lire le HTML local ({}): {} — bascule vers URL {}", path, e, url);
                builder.with_url(url)
            }
        }
    } else {
        builder.with_url(url)
    };
    let _webview = match builder
        .with_devtools(std::env::var("READRSS_WEBVIEW_DEVTOOLS").is_ok())
        .build()
    {
        Ok(wv) => wv,
        Err(e) => { eprintln!("Impossible de créer la WebView: {}", e); return; }
    };

    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => { *control_flow = ControlFlow::Exit; }
            _ => {}
        }
    });
}
