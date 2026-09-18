mod auth;
mod polkit_auth;
mod profile;
mod ui;
mod vpn;

use adw::prelude::*;
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;

use profile::{load_profiles, unique_profile};
use ui::{
    clear_editor, create_ui, load_selected_profile, refresh_profile_list, save_current_profile,
    save_profiles_or_dialog, Ui,
};
use vpn::{start_connection, stop_connection, update_actions, AppState};

const APP_ID: &str = "org.openfortigui.Gtk";

fn main() -> glib::ExitCode {
    let application = adw::Application::builder().application_id(APP_ID).build();
    application.connect_activate(build_window);
    application.run()
}

fn build_window(app: &adw::Application) {
    adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default);

    let state = Rc::new(RefCell::new(AppState {
        profiles: load_profiles(),
        ..AppState::default()
    }));
    let ui = Rc::new(create_ui(app));

    refresh_profile_list(&ui, &state);
    clear_editor(&ui);
    update_actions(&ui, &state);
    wire_actions(&ui, &state);

    // Authentication will be requested on each connect/disconnect action.

    ui.window.present();
}

fn wire_actions(ui: &Rc<Ui>, state: &Rc<RefCell<AppState>>) {
    let ui_ref = Rc::clone(ui);
    let state_ref = Rc::clone(state);
    ui.add_button.connect_clicked(move |_| {
        let mut state = state_ref.borrow_mut();
        let profile = unique_profile(&state.profiles);
        state.profiles.push(profile);
        state.selected = Some(state.profiles.len() - 1);
        drop(state);
        refresh_profile_list(&ui_ref, &state_ref);
        load_selected_profile(&ui_ref, &state_ref);
        save_profiles_or_dialog(&ui_ref, &state_ref);
        update_actions(&ui_ref, &state_ref);
    });

    let ui_ref = Rc::clone(ui);
    let state_ref = Rc::clone(state);
    ui.delete_button.connect_clicked(move |_| {
        let Some(index) = state_ref.borrow().selected else {
            return;
        };
        state_ref.borrow_mut().profiles.remove(index);
        state_ref.borrow_mut().selected = None;
        refresh_profile_list(&ui_ref, &state_ref);
        clear_editor(&ui_ref);
        save_profiles_or_dialog(&ui_ref, &state_ref);
        update_actions(&ui_ref, &state_ref);
    });

    let ui_ref = Rc::clone(ui);
    let state_ref = Rc::clone(state);
    ui.profile_list.connect_row_selected(move |_, row| {
        state_ref.borrow_mut().selected = row.map(|row| row.index() as usize);
        load_selected_profile(&ui_ref, &state_ref);
        update_actions(&ui_ref, &state_ref);
    });

    let ui_ref = Rc::clone(ui);
    let state_ref = Rc::clone(state);
    ui.save_button.connect_clicked(move |_| {
        save_current_profile(&ui_ref, &state_ref);
        refresh_profile_list(&ui_ref, &state_ref);
        update_actions(&ui_ref, &state_ref);
    });

    let ui_ref = Rc::clone(ui);
    let state_ref = Rc::clone(state);
    ui.connect_button.connect_clicked(move |_| {
        save_current_profile(&ui_ref, &state_ref);
        start_connection(&ui_ref, &state_ref);
    });

    let ui_ref = Rc::clone(ui);
    let state_ref = Rc::clone(state);
    ui.disconnect_button.connect_clicked(move |_| {
        stop_connection(&ui_ref, &state_ref);
    });
}
