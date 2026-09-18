use gtk::glib;
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Child;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use adw::prelude::*;
use crate::auth::open_saml_login;
use crate::polkit_auth::{start_vpn_privileged, stop_vpn_privileged};
use crate::profile::{write_runtime_config, VpnProfile};
use crate::ui::{append_log, save_profiles_or_dialog, show_error, Ui};

#[derive(Debug)]
pub enum ProcessEvent {
    Output(String),
    Finished(i32),
}

#[derive(Debug, Default)]
pub struct AppState {
    pub profiles: Vec<VpnProfile>,
    pub selected: Option<usize>,
    pub child: Option<Child>,
    pub runtime_config: Option<PathBuf>,
    pub receiver: Option<Receiver<ProcessEvent>>,
    pub saml_opened: bool,
    pub pending_cert_prompts: HashSet<String>,
}

pub fn start_connection(ui: &Rc<Ui>, state: &Rc<RefCell<AppState>>) {
    if state.borrow().child.is_some() {
        append_log(ui, "Ja existe uma conexao em andamento.\n");
        return;
    }

    let Some(profile) = ({
        let state = state.borrow();
        state
            .selected
            .and_then(|index| state.profiles.get(index).cloned())
    })
    else {
        return;
    };

    if profile.gateway_host.trim().is_empty() {
        show_error(ui, "Perfil incompleto", "Informe o gateway antes de conectar.");
        return;
    }

    ui.log_buffer.set_text("");
    append_log(ui, "Iniciando openfortivpn com privilegios administrativos...\n");

    let runtime_config = match write_runtime_config(&profile) {
        Ok(path) => path,
        Err(error) => {
            show_error(
                ui,
                "Nao foi possivel preparar a VPN",
                &format!("Falha ao criar configuracao temporaria: {error}"),
            );
            return;
        }
    };

    match start_vpn_privileged(&runtime_config) {
        Ok(mut child) => {
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();
            let (sender, receiver) = mpsc::channel();
            pipe_output(stdout, sender.clone());
            pipe_output(stderr, sender);

            let mut state_mut = state.borrow_mut();
            state_mut.child = Some(child);
            state_mut.runtime_config = Some(runtime_config);
            state_mut.receiver = Some(receiver);
            state_mut.saml_opened = false;
            state_mut.pending_cert_prompts.clear();
            drop(state_mut);

            update_actions(ui, state);
            poll_process(ui, state, profile);
        }
        Err(error) => {
            let _ = fs::remove_file(&runtime_config);
            show_error(
                ui,
                "Nao foi possivel iniciar a VPN",
                &format!("Falha ao executar pkexec/openfortivpn: {error}"),
            );
        }
    }
}

pub fn stop_connection(ui: &Ui, state: &Rc<RefCell<AppState>>) {
    let state_ref = state.borrow();
    let Some(child) = &state_ref.child else {
        return;
    };
    let pid = child.id();
    match stop_vpn_privileged(pid) {
        Ok(_) => append_log(ui, "Desconexao solicitada com privilegios administrativos.\n"),
        Err(error) => append_log(ui, &format!("Falha ao solicitar desconexao como root: {error}\n")),
    }
}

fn poll_process(ui: &Rc<Ui>, state: &Rc<RefCell<AppState>>, profile: VpnProfile) {
    let ui_ref = Rc::clone(ui);
    let state_ref = Rc::clone(state);
    glib::timeout_add_local(Duration::from_millis(150), move || {
        let mut events = Vec::new();
        if let Some(receiver) = &state_ref.borrow().receiver {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }

        for event in events {
            match event {
                ProcessEvent::Output(line) => handle_output(&ui_ref, &state_ref, &profile, &line),
                ProcessEvent::Finished(code) => {
                    append_log(&ui_ref, &format!("openfortivpn finalizou com codigo {code}.\n"));
                }
            }
        }

        let exited = state_ref
            .borrow_mut()
            .child
            .as_mut()
            .and_then(|child| child.try_wait().ok().flatten())
            .map(|status| status.code().unwrap_or(-1));

        if let Some(code) = exited {
            append_log(&ui_ref, &format!("Processo encerrado com codigo {code}.\n"));
            let mut state = state_ref.borrow_mut();
            state.child = None;
            remove_runtime_config(&mut state);
            state.receiver = None;
            drop(state);
            update_actions(&ui_ref, &state_ref);
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn handle_output(ui: &Rc<Ui>, state: &Rc<RefCell<AppState>>, profile: &VpnProfile, line: &str) {
    append_log(ui, line);
    append_log(ui, "\n");

    if profile.saml_login && line.contains("Authenticate at '") && !state.borrow().saml_opened {
        state.borrow_mut().saml_opened = true;
        open_saml_login(ui, profile);
    }

    if let Some(cert) = extract_trusted_cert(line) {
        let should_ask = {
            let mut state = state.borrow_mut();
            let already_saved = state
                .selected
                .and_then(|index| state.profiles.get(index))
                .is_some_and(|selected_profile| selected_profile.trusted_cert.trim() == cert);
            !already_saved && state.pending_cert_prompts.insert(cert.clone())
        };

        if should_ask {
            ask_to_trust_certificate(ui, state, cert, extract_certificate_details(line));
        }
    }
}

fn pipe_output(pipe: Option<impl std::io::Read + Send + 'static>, sender: Sender<ProcessEvent>) {
    if let Some(pipe) = pipe {
        std::thread::spawn(move || {
            for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                let _ = sender.send(ProcessEvent::Output(line));
            }
            let _ = sender.send(ProcessEvent::Finished(0));
        });
    }
}

fn ask_to_trust_certificate(ui: &Rc<Ui>, state: &Rc<RefCell<AppState>>, cert: String, details: String) {
    let body = if details.is_empty() {
        format!("O gateway retornou um certificado nao confiavel.\n\nSHA256: {cert}\n\nAdicionar ao perfil e salvar?")
    } else {
        format!("O gateway retornou um certificado nao confiavel.\n\n{details}\nSHA256: {cert}\n\nAdicionar ao perfil e salvar?")
    };

    let dialog = adw::MessageDialog::builder()
        .transient_for(&ui.window)
        .heading("Certificado do gateway")
        .body(&body)
        .build();
    dialog.add_response("cancel", "Cancelar");
    dialog.add_response("trust", "Adicionar");
    dialog.set_response_appearance("trust", adw::ResponseAppearance::Suggested);

    let ui_ref = Rc::clone(ui);
    let state_ref = Rc::clone(state);
    dialog.connect_response(None, move |dialog, response| {
        if response == "trust" {
            let selected = state_ref.borrow().selected;
            if let Some(index) = selected {
                {
                    let mut state = state_ref.borrow_mut();
                    if let Some(profile) = state.profiles.get_mut(index) {
                        profile.trusted_cert = cert.clone();
                    }
                }
                ui_ref.trusted_cert.set_text(&cert);
                save_profiles_or_dialog(&ui_ref, &state_ref);
                append_log(&ui_ref, "Certificado salvo no perfil. Conecte novamente para usa-lo.\n");
            }
        }
        dialog.close();
    });
    dialog.present();
}

fn extract_trusted_cert(line: &str) -> Option<String> {
    let regex = Regex::new(r"--trusted-cert[= ]([A-Fa-f0-9:]+)").ok()?;
    regex
        .captures(line)
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
}

fn extract_certificate_details(line: &str) -> String {
    let Some(start) = line.find("Gateway certificate:") else {
        return String::new();
    };
    let details = &line[start..];
    details
        .split("sha256 digest:")
        .next()
        .unwrap_or(details)
        .replace("ERROR:", "")
        .trim()
        .to_string()
}

fn remove_runtime_config(state: &mut AppState) {
    if let Some(path) = state.runtime_config.take() {
        let _ = fs::remove_file(path);
    }
}

pub fn update_actions(ui: &Ui, state: &Rc<RefCell<AppState>>) {
    let has_selection = state.borrow().selected.is_some();
    let running = state.borrow().child.is_some();
    ui.delete_button.set_sensitive(has_selection && !running);
    ui.save_button.set_sensitive(has_selection && !running);
    ui.connect_button.set_sensitive(has_selection && !running);
    ui.disconnect_button.set_sensitive(running);
}
