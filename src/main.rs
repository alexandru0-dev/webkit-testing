use gtk4::{
    gio::ApplicationFlags,
    glib::{self, Bytes},
    prelude::*,
    ApplicationWindow, PageSetup, PrintSettings, Window,
};
use webkit6::{prelude::*, LoadEvent, PrintOperation, WebView};

use std::borrow::Borrow;
use std::time::Instant;
use uuid::Uuid;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router, ServiceExt,
};

use axum::extract::Query;
use serde::Deserialize;
use std::sync::OnceLock;
use tokio::runtime;

use std::sync::{Arc, Mutex};

use tokio::sync::{mpsc, oneshot};

#[derive(Deserialize)]
struct QParam {
    id: u32,
}

fn runtime() -> &'static runtime::Runtime {
    static RUNTIME: OnceLock<runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_io()
            .build()
            .expect("Setting up tokio runtime needs to succeed.")
    })
}

fn build_print(wv: &WebView, id: &u32) -> PrintOperation {
    let settings = PrintSettings::new();
    settings.load_file("./settings.txt").unwrap();
    let v7 = Uuid::now_v7();
    settings.set(
        "output-uri",
        Some(&format!(
            "file:///home/alex0/pdf_test/multiple_{}_{}.pdf",
            id, v7
        )),
    );
    //settings.set_output_bin("./here.pdf");
    //settings.set_output_bin(&format!("/home/alex0/multiple_{}.pdf", id));
    //y.to_file("./test.pdf").unwrap();

    let pagesetup = PageSetup::from_file("./settings.txt").unwrap();
    PrintOperation::builder()
        .print_settings(&settings)
        .web_view(wv)
        .page_setup(&pagesetup)
        .build()
}

fn webview_load_changed(
    _wv: &WebView,
    event: LoadEvent,
    mut tx: std::sync::MutexGuard<Option<oneshot::Sender<u32>>>,
) {
    match event {
        LoadEvent::Finished => println!("finished"),
        _ => (),
    }
}

const HTML: &[u8] = include_bytes!("./odyssey.html");
//const ODYSSEY: Bytes = Bytes::from_static(HTML);

async fn root(Query(param): Query<QParam>) -> &'static str {
    let (print_tx, print_rx) = oneshot::channel::<u32>();
    let dio: Arc<Mutex<Option<oneshot::Sender<u32>>>> = Arc::new(Mutex::new(Some(print_tx)));

    gio::gtk4::gio::spawn_blocking(async move || {
        let (window, webview) = build_webview().await;

        webview.connect_print(|_webview, _op: &PrintOperation| true);

        webview.connect_load_changed(move |a, b| {
            let x = dio.lock().unwrap();
            webview_load_changed(a, b, x);
        });

        webview.load_bytes(&Bytes::from_static(HTML), Some("text/html"), None, None);
        //webview.load_html(&format!("<p>testing id: {}</p>", id), None);

        webview.connect_close(glib::clone!(
            #[weak]
            window,
            move |x| {
                println!("closed {:?}", x);
                //window
                window.close();
                window.destroy();
            },
        ));
    });
    println!("param.id : {}", param.id);
    //state.tx.send(param.id).unwrap();
    "Hello, World!"
}

fn main() {
    let runtime = runtime();
    runtime.spawn(async move {
        let app = Router::new().route("/", get(root));
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    //let flags = ApplicationFlags::IS_LAUNCHER;

    let app = gtk4::Application::new(Some("org.gnome.webkit6-rs.dio"), Default::default());

    app.connect_activate(build_ui);
    app.run();
}

enum Command {
    A,
    // Other commands can be added here
}

fn build_ui(app: &gtk4::Application) {
    let _ = ApplicationWindow::new(app);
}

async fn build_webview() -> (Window, WebView) {
    let window = Window::new();
    let settings = webkit6::Settings::builder().build();
    settings.set_enable_javascript(false);
    settings.set_enable_page_cache(false);
    settings.set_enable_html5_database(false);
    settings.set_enable_html5_local_storage(false);
    settings.set_disable_web_security(false);
    settings.set_enable_page_cache(false);
    settings.set_enable_back_forward_navigation_gestures(false);

    let webview = WebView::builder().settings(&settings).build();
    //webview.set_settings(&settings);

    //webview.settings().set_gtk_enable_accels(false);

    window.set_child(Some(&webview));

    (window, webview)
}
