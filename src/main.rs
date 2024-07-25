use gtk4::{glib, prelude::*, ApplicationWindow, PrintSettings};
use webkit6::{prelude::*, PrintOperation, WebView};

fn main() -> glib::ExitCode {
    let app = gtk4::Application::new(Some("org.gnome.webkit6-rs.example"), Default::default());
    app.connect_activate(move |app| {
        let window = ApplicationWindow::new(app);
        let webview = WebView::new();

        webview.load_uri("https://crates.io/");
        window.set_child(Some(&webview));

        let y = PrintSettings::new();
        y.set_printer("Print to File");
        y.to_file("./test.pdf").unwrap();

        let x = PrintOperation::new(&webview);
        x.set_print_settings(&y);
        //x.print();
        //println!("{:?}", x.run_dialog(Some(&window)));

        let settings = WebViewExt::settings(&webview).unwrap();
        settings.set_enable_developer_extras(true);

        let inspector = webview.inspector().unwrap();
        inspector.show();

        webview.evaluate_javascript("42", None, None, gtk4::gio::Cancellable::NONE, |result| {
            match result {
                Ok(value) => {
                    println!("is_boolean: {}", value.is_boolean());
                    println!("is_number: {}", value.is_number());
                    println!("{:?}", value.to_boolean());
                }
                Err(error) => println!("{}", error),
            }
        });
        window.present();
    });
    app.run()
}
