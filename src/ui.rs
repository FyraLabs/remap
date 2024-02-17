use libhelium::{
    glib::{self, signal_stop_emission_by_name},
    gtk::{self, traits::ButtonExt, Editable, Entry, EventControllerKey, Notebook, ToggleButton},
    prelude::{BoxExt, EditableExt, EditableExtManual, GtkWindowExt, ToggleButtonExt, WidgetExt},
    Application, ApplicationWindow, FillButton,
};

pub struct ShortcutEntry {
    pub entry: Entry,
}

impl ShortcutEntry {
    fn new() -> Self {
        let entry = Entry::new();
        let editable = entry.delegate().unwrap();
        editable.connect_insert_text(Self::on_insert_text);
        let ctl = EventControllerKey::new();
        ctl.connect_key_pressed(Self::on_keypress);
        entry.add_controller(ctl);
        Self { entry }
    }

    fn on_insert_text(this: &Editable, text: &str, pos: &mut i32) {
        signal_stop_emission_by_name(this, "insert-text");
        println!("insert: {text:?} {pos}");
    }

    fn on_keypress(
        this: &EventControllerKey,
        key: libhelium::gdk::Key,
        idk: u32,
        modt: libhelium::gdk::ModifierType,
    ) -> glib::Propagation {
        println!("{this:#?} {key:#?} {idk} {modt:#?}");
        glib::Propagation::Stop
    }
}

pub trait RemapPage {
    const PAGE_NAME: &'static str;

    fn tab_label() -> gtk::Label {
        gtk::Label::new(Some(Self::PAGE_NAME))
    }

    fn page(&self) -> &gtk::ListBox;
}

pub struct SimpleRemap {
    pub page: gtk::ListBox,
    pub tab_label: gtk::Label,
}

impl RemapPage for SimpleRemap {
    const PAGE_NAME: &'static str = "Simple";

    fn page(&self) -> &gtk::ListBox {
        &self.page
    }
}

impl SimpleRemap {
    pub fn new(nb: &Notebook) -> Self {
        let ret = Self {
            page: gtk::ListBox::new(),
            tab_label: Self::tab_label(),
        };
        nb.append_page(&ret.page, Some(&ret.tab_label));
        ret
    }
}

pub struct MainWindow {
    pub win: ApplicationWindow,
    pub vertbox: gtk::Box,
    pub stack: gtk::Stack,
    pub page: gtk::Box,
    pub reloadbtn: FillButton,
    pub savebtn: FillButton,
    pub toggle: ToggleButton,
}

impl MainWindow {
    pub fn box_content() -> gtk::Box {
        let b = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            // .spacing(20)
            .build();
        b.append(&Self::app_bar());
        b
    }

    pub fn make_fillbtn(label: &str, f: impl Fn(&FillButton) + 'static) -> FillButton {
        let btn = FillButton::new(label);
        btn.connect_clicked(f);
        btn
    }

    pub fn on_toggle(btn: &ToggleButton) {
        // TODO: actually do something
        btn.toggled();
        if btn.is_active() {
            btn.set_label("Disable keyd");
        } else {
            btn.set_label("Enable keyd");
        }
    }
    pub fn on_reload(btn: &FillButton) {
        todo!()
    }
    pub fn on_save(btn: &FillButton) {
        todo!()
    }

    pub fn bottom_btnbar() -> (gtk::Box, FillButton, FillButton, ToggleButton) {
        let b = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .halign(gtk::Align::End)
            .valign(gtk::Align::End)
            .margin_end(15)
            .margin_bottom(20)
            .build();
        let toggle = ToggleButton::new();
        toggle.set_label("Enable keyd");
        toggle.connect_clicked(Self::on_toggle);
        // TODO: add code to detect if keyd is running
        b.append(&toggle);
        let (reloadbtn, savebtn) = (
            Self::make_fillbtn("Reload", Self::on_reload),
            Self::make_fillbtn("Save", Self::on_save),
        );
        b.append(&reloadbtn);
        b.append(&savebtn);
        (b, reloadbtn, savebtn, toggle)
    }

    pub fn app_bar() -> libhelium::AppBar {
        let bar = libhelium::AppBar::builder()
            .viewtitle_widget(
                &libhelium::gtk::Label::builder()
                    .label("Remap")
                    .halign(gtk::Align::Start)
                    .lines(1)
                    .css_classes(vec!["view-title"])
                    .build(),
            )
            .build();
        bar
    }

    pub fn new(app: &Application) -> Self {
        let b = Self::box_content();
        let nb = Notebook::builder()
            .show_border(false)
            .tab_pos(gtk::PositionType::Left)
            .build();
        nb.set_hexpand(true);
        nb.set_vexpand(true);
        SimpleRemap::new(&nb);
        b.append(&nb);
        let win = ApplicationWindow::builder()
            .application(app)
            .child(&b)
            .title("Remap")
            .visible(true)
            // .has_title(true)
            .default_width(1000)
            .default_height(800)
            .mnemonics_visible(true)
            .hexpand(true)
            .vexpand(true)
            .build();
        let (bottom_btnbar, reloadbtn, savebtn, toggle) = Self::bottom_btnbar();
        b.append(&bottom_btnbar);

        win.present();

        // Self {
        //     win,
        //     nb,
        //     reloadbtn,
        //     savebtn,
        //     toggle,
        // }
        todo!()
    }
}
