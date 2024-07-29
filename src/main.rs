use gtk4::{
    glib::{self, Bytes},
    prelude::*,
    ApplicationWindow, PageSetup, PrintSettings, Window,
};
use webkit6::{prelude::*, LoadEvent, PrintOperation, WebView};

use uuid::Uuid;

use axum::{
    body::Body,
    extract::Query,
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse},
    routing::get,
    Router,
};

use tokio::{fs::File, sync::broadcast::Sender};
use tokio_util::io::ReaderStream;

use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Deserialize)]
struct QParam {
    id: u32,
}

fn build_print(wv: &WebView, id: Uuid) {
    let settings = PrintSettings::new();
    settings.load_file("./settings.txt").unwrap_or_default();
    settings.set(
        "output-uri",
        Some(&format!("file:///tmp/wk_print_{}.pdf", id)),
    );

    let pagesetup = PageSetup::from_file("./settings.txt").unwrap_or_default();
    let print_op = PrintOperation::builder()
        .print_settings(&settings)
        .web_view(wv)
        .page_setup(&pagesetup)
        .build();

    let cln = wv.clone();

    print_op.connect_finished(move |_print_op: &PrintOperation| {
        sender().send(id).unwrap();
        println!("{} saved successfully ", id);
        cln.try_close();
    });

    print_op.print();
}

fn webview_load_changed(wv: &WebView, event: LoadEvent, id: Uuid) {
    match event {
        LoadEvent::Finished => {
            build_print(wv, id);
        }
        _ => (),
    }
}

const HTML: &str = include_str!("./assets/weasyprint-samples/poster/poster.html");

async fn root(Query(param): Query<QParam>) -> Result<impl IntoResponse, ()> {
    let v7 = Uuid::now_v7();

    glib::spawn_future(async move {
        load_webview(&v7);
    });

    let mut rx = sender().subscribe();

    let mut a = rx.recv().await.unwrap();
    while a != v7 {
        a = rx.recv().await.unwrap();
    }

    let file = match File::open(format!("/tmp/wk_print_{}.pdf", v7)).await {
        Ok(file) => file,
        Err(err) => {
            println!("Error: {:?}", err);
            todo!();
        }
    };
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/pdf charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"test.pdf\"",
        ),
    ];

    Ok((headers, body))
}

fn gtk_main() {
    let app = gtk4::Application::new(Some("org.gnome.webkit6-rs.dio"), Default::default());
    app.connect_activate(move |app: &gtk4::Application| {
        build_ui(app);
    });
    app.run();
}

fn sender() -> &'static Sender<Uuid> {
    static RUNTIME: OnceLock<Sender<Uuid>> = OnceLock::new();
    RUNTIME.get_or_init(|| Sender::new(100))
}

#[tokio::main]
async fn main() {
    std::thread::spawn(gtk_main);

    let app = Router::new().route("/", get(root));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn build_ui(app: &gtk4::Application) {
    let _ = ApplicationWindow::new(app);
}

fn load_webview(id: &Uuid) {
    let (window, webview) = build_webview();
    //window.show();
    webview.connect_print(|_webview, _print_op| true);

    let id_copy = id.clone();

    webview.connect_load_changed(move |a, b| {
        webview_load_changed(a, b, id_copy);
    });

    webview.load_html(
        &HTML,
        Some(
            "file:///home/alex0/Repos/Personal/webkit-testing/src/assets/weasyprint-samples/poster/",
        ),
    );
    webview.connect_close(glib::clone!(
        #[weak]
        window,
        move |x| {
            println!("closed {:?}", x);
            window.destroy();
        },
    ));
}

fn build_webview() -> (Window, WebView) {
    let settings = webkit6::Settings::builder()
        .enable_javascript(true)
        .enable_media(true)
        .print_backgrounds(true)
        .enable_page_cache(false)
        .enable_html5_database(false)
        .enable_html5_local_storage(false)
        .disable_web_security(true)
        .enable_back_forward_navigation_gestures(false)
        .build();

    let webview = WebView::builder().settings(&settings).build();
    let window = Window::builder()
        .default_width(1920)
        .default_height(1080)
        .child(&webview)
        .build();

    (window, webview)
}
