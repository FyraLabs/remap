mod macroparse;
pub mod parse;
mod ui;

use libhelium::{
    glib,
    prelude::{ApplicationExt, ApplicationExtManual},
    Application,
};

const APP_ID: &str = "com.fyralabs.remap";

struct Remap {
    pub app: Application,
}

impl Remap {
    fn new() -> Self {
        let app = Application::builder().application_id(APP_ID).build();
        Self { app }
    }
    fn run(&self) -> glib::ExitCode {
        self.app.run()
    }
}

fn main() -> glib::ExitCode {
    libhelium::gtk::init().unwrap();
    let app = Remap::new();
    app.app.connect_activate(|app| {
        ui::MainWindow::new(&app);
    });
    app.run()
}
