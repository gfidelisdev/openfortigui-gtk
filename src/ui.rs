use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::profile::{save_profiles, VpnProfile};
use crate::vpn::AppState;

pub struct Ui {
    pub window: adw::ApplicationWindow,
    pub add_button: gtk::Button,
    pub delete_button: gtk::Button,
    pub profile_list: gtk::ListBox,
    pub name: gtk::Entry,
    pub gateway_host: gtk::Entry,
    pub gateway_port: gtk::SpinButton,
    pub username: gtk::Entry,
    pub password: gtk::PasswordEntry,
    pub realm: gtk::Entry,
    pub trusted_cert: gtk::Entry,
    pub saml_login: gtk::Switch,
    pub saml_port: gtk::SpinButton,
    pub set_routes: gtk::Switch,
    pub set_dns: gtk::Switch,
    pub persistent: gtk::Switch,
    pub save_button: gtk::Button,
    pub connect_button: gtk::Button,
    pub disconnect_button: gtk::Button,
    pub log_buffer: gtk::TextBuffer,
    pub log_view: gtk::TextView,
    pub log_auto_scroll: std::rc::Rc<std::cell::RefCell<bool>>,
}

pub fn create_ui(app: &adw::Application) -> Ui {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("OpenFortiGUI GTK")
        .default_width(1040)
        .default_height(840)
        .build();

    let header = adw::HeaderBar::new();
    let add_button = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Adicionar perfil")
        .build();
    let delete_button = gtk::Button::builder()
        .icon_name("user-trash-symbolic")
        .tooltip_text("Remover perfil")
        .build();
    header.pack_start(&add_button);
    header.pack_start(&delete_button);

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    window.set_content(Some(&toolbar_view));

    let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    root.add_css_class("view");
    toolbar_view.set_content(Some(&root));

    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
    sidebar.set_width_request(280);
    sidebar.add_css_class("sidebar");
    root.append(&sidebar);

    let sidebar_title = gtk::Label::new(Some("Perfis VPN"));
    sidebar_title.set_xalign(0.0);
    sidebar_title.add_css_class("heading");
    sidebar_title.set_margin_top(18);
    sidebar_title.set_margin_bottom(12);
    sidebar_title.set_margin_start(18);
    sidebar_title.set_margin_end(18);
    sidebar.append(&sidebar_title);

    let profile_list = gtk::ListBox::new();
    profile_list.set_selection_mode(gtk::SelectionMode::Single);
    profile_list.add_css_class("navigation-sidebar");
    sidebar.append(&profile_list);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.set_hexpand(true);
    root.append(&content);

    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    toolbar.set_margin_top(18);
    toolbar.set_margin_bottom(12);
    toolbar.set_margin_start(18);
    toolbar.set_margin_end(18);
    content.append(&toolbar);

    let title = gtk::Label::new(Some("Conexão Fortinet"));
    title.set_xalign(0.0);
    title.set_hexpand(true);
    title.add_css_class("title-2");
    toolbar.append(&title);

    let save_button = gtk::Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text("Salvar perfil")
        .build();
    toolbar.append(&save_button);

    let connect_button = gtk::Button::with_label("Conectar");
    connect_button.add_css_class("suggested-action");
    toolbar.append(&connect_button);

    let disconnect_button = gtk::Button::with_label("Desconectar");
    disconnect_button.add_css_class("destructive-action");
    toolbar.append(&disconnect_button);

    let paned = gtk::Paned::new(gtk::Orientation::Vertical);
    paned.set_hexpand(true);
    paned.set_vexpand(true);
    paned.set_position(430);
    content.append(&paned);

    let form = gtk::Grid::builder()
        .column_spacing(14)
        .row_spacing(12)
        .margin_top(6)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build();
    form.set_hexpand(true);
    form.set_vexpand(false);
    form.set_valign(gtk::Align::Start);
    let form_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&form)
        .build();
    form_scroll.set_hexpand(true);
    form_scroll.set_vexpand(true);
    paned.set_start_child(Some(&form_scroll));

    let name = gtk::Entry::new();
    let gateway_host = gtk::Entry::new();
    let gateway_port = gtk::SpinButton::with_range(1.0, 65535.0, 1.0);
    let username = gtk::Entry::new();
    let password = gtk::PasswordEntry::new();
    let realm = gtk::Entry::new();
    let trusted_cert = gtk::Entry::new();
    let saml_login = gtk::Switch::new();
    let saml_port = gtk::SpinButton::with_range(1.0, 65535.0, 1.0);
    let set_routes = gtk::Switch::new();
    let set_dns = gtk::Switch::new();
    let persistent = gtk::Switch::new();

    attach_row(&form, 0, "Nome", &name);
    attach_row(&form, 1, "Gateway", &gateway_host);
    attach_row(&form, 2, "Porta", &gateway_port);
    attach_row(&form, 3, "Usuário", &username);
    attach_row(&form, 4, "Senha", &password);
    attach_row(&form, 5, "Realm", &realm);
    attach_row(&form, 6, "Trusted cert SHA256", &trusted_cert);
    attach_row(&form, 7, "Login SAML", &saml_login);
    attach_row(&form, 8, "Porta SAML", &saml_port);
    attach_row(&form, 9, "Configurar rotas", &set_routes);
    attach_row(&form, 10, "Configurar DNS", &set_dns);
    attach_row(&form, 11, "Persistente", &persistent);
    keep_switch_compact(&saml_login);
    keep_switch_compact(&set_routes);
    keep_switch_compact(&set_dns);
    keep_switch_compact(&persistent);

    let log_view = gtk::TextView::new();
    log_view.set_editable(false);
    log_view.set_monospace(true);
    log_view.set_wrap_mode(gtk::WrapMode::WordChar);
    let log_buffer = log_view.buffer();
    let log_scroll = gtk::ScrolledWindow::builder()
        .min_content_height(180)
        .child(&log_view)
        .build();
    log_scroll.set_margin_top(6);
    log_scroll.set_margin_bottom(18);
    log_scroll.set_margin_start(18);
    log_scroll.set_margin_end(18);
    paned.set_end_child(Some(&log_scroll));

    // Auto-scroll behaviour: keep scrolling to end unless user scrolls manually.
    let log_auto_scroll = Rc::new(RefCell::new(true));
    {
        let log_auto_scroll = Rc::clone(&log_auto_scroll);
        let adj = log_scroll.vadjustment();
        adj.connect_value_changed(move |_| {
            *log_auto_scroll.borrow_mut() = false;
        });
    }

    Ui {
        window,
        add_button,
        delete_button,
        profile_list,
        name,
        gateway_host,
        gateway_port,
        username,
        password,
        realm,
        trusted_cert,
        saml_login,
        saml_port,
        set_routes,
        set_dns,
        persistent,
        save_button,
        connect_button,
        disconnect_button,
        log_buffer,
        log_view,
        log_auto_scroll,
    }
}

fn attach_row<W: IsA<gtk::Widget>>(grid: &gtk::Grid, row: i32, label: &str, widget: &W) {
    let row_label = gtk::Label::new(Some(label));
    row_label.set_xalign(0.0);
    row_label.set_halign(gtk::Align::Start);
    row_label.add_css_class("dim-label");
    grid.attach(&row_label, 0, row, 1, 1);
    widget.set_hexpand(true);
    widget.set_halign(gtk::Align::Fill);
    grid.attach(widget, 1, row, 1, 1);
}

fn keep_switch_compact(switch: &gtk::Switch) {
    switch.set_hexpand(false);
    switch.set_halign(gtk::Align::Start);
}

pub fn refresh_profile_list(ui: &Ui, state: &Rc<RefCell<AppState>>) {
    let (profiles, selected) = {
        let state = state.borrow();
        (state.profiles.clone(), state.selected)
    };

    while let Some(row) = ui.profile_list.first_child() {
        ui.profile_list.remove(&row);
    }

    for profile in &profiles {
        let row = gtk::ListBoxRow::new();
        let label = gtk::Label::new(Some(&profile.name));
        label.set_xalign(0.0);
        label.set_margin_top(10);
        label.set_margin_bottom(10);
        label.set_margin_start(14);
        label.set_margin_end(14);
        row.set_child(Some(&label));
        ui.profile_list.append(&row);
    }

    if let Some(index) = selected {
        if let Some(row) = ui.profile_list.row_at_index(index as i32) {
            ui.profile_list.select_row(Some(&row));
        }
    }
}

pub fn load_selected_profile(ui: &Ui, state: &Rc<RefCell<AppState>>) {
    let profile = {
        let state = state.borrow();
        state
            .selected
            .and_then(|index| state.profiles.get(index).cloned())
    };

    if let Some(profile) = profile {
        ui.name.set_text(&profile.name);
        ui.gateway_host.set_text(&profile.gateway_host);
        ui.gateway_port.set_value(profile.gateway_port as f64);
        ui.username.set_text(&profile.username);
        ui.password.set_text(&profile.password);
        ui.realm.set_text(&profile.realm);
        ui.trusted_cert.set_text(&profile.trusted_cert);
        ui.saml_login.set_active(profile.saml_login);
        ui.saml_port.set_value(profile.saml_port as f64);
        ui.set_routes.set_active(profile.set_routes);
        ui.set_dns.set_active(profile.set_dns);
        ui.persistent.set_active(profile.persistent);
    } else {
        clear_editor(ui);
    }
}

pub fn clear_editor(ui: &Ui) {
    ui.name.set_text("");
    ui.gateway_host.set_text("");
    ui.gateway_port.set_value(443.0);
    ui.username.set_text("");
    ui.password.set_text("");
    ui.realm.set_text("");
    ui.trusted_cert.set_text("");
    ui.saml_login.set_active(false);
    ui.saml_port.set_value(8020.0);
    ui.set_routes.set_active(true);
    ui.set_dns.set_active(true);
    ui.persistent.set_active(false);
}

pub fn save_current_profile(ui: &Ui, state: &Rc<RefCell<AppState>>) {
    let Some(index) = state.borrow().selected else {
        return;
    };

    let profile = VpnProfile {
        name: text(&ui.name),
        gateway_host: text(&ui.gateway_host),
        gateway_port: ui.gateway_port.value_as_int() as u16,
        username: text(&ui.username),
        password: ui.password.text().to_string(),
        realm: text(&ui.realm),
        trusted_cert: text(&ui.trusted_cert),
        saml_login: ui.saml_login.is_active(),
        saml_port: ui.saml_port.value_as_int() as u16,
        set_routes: ui.set_routes.is_active(),
        set_dns: ui.set_dns.is_active(),
        persistent: ui.persistent.is_active(),
    };

    state.borrow_mut().profiles[index] = profile;
    save_profiles_or_dialog(ui, state);
}

pub fn save_profiles_or_dialog(ui: &Ui, state: &Rc<RefCell<AppState>>) {
    if let Err(error) = save_profiles(&state.borrow().profiles) {
        show_error(ui, "Nao foi possivel salvar", &error.to_string());
    }
}

pub fn append_log(ui: &Ui, text: &str) {
    let mut end = ui.log_buffer.end_iter();
    ui.log_buffer.insert(&mut end, text);

    // Scroll to end only if auto-scroll is enabled.
    if *ui.log_auto_scroll.borrow() {
        let mark = ui.log_buffer.create_mark(None, &ui.log_buffer.end_iter(), false);
        ui.log_view.scroll_to_mark(&mark, 0.0, false, 0.0, 1.0);
        ui.log_buffer.delete_mark(&mark);
    }
}

pub fn show_error(ui: &Ui, heading: &str, body: &str) {
    let dialog = adw::MessageDialog::builder()
        .transient_for(&ui.window)
        .heading(heading)
        .body(body)
        .build();
    dialog.add_response("ok", "OK");
    dialog.present();
}

fn text(entry: &gtk::Entry) -> String {
    entry.text().trim().to_string()
}
