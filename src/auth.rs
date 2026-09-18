use gtk::gio;

use crate::profile::VpnProfile;
use crate::ui::{append_log, show_error, Ui};

/// Rebuilds the SAML URL from the profile (never trusts anything scraped from process output) and opens it in the user's own browser.
pub fn open_saml_login(ui: &Ui, profile: &VpnProfile) {
    let mut url = format!(
        "https://{}:{}/remote/saml/start?redirect=1",
        profile.gateway_host.trim(),
        profile.gateway_port
    );
    if !profile.realm.trim().is_empty() {
        url.push_str("&realm=");
        url.push_str(&urlencoding::encode(profile.realm.trim()));
    }

    append_log(ui, &format!("Abrindo login SAML no navegador: {url}\n"));
    if let Err(error) = gio::AppInfo::launch_default_for_uri(&url, None::<&gio::AppLaunchContext>) {
        show_error(
            ui,
            "Abra o login SAML no navegador",
            &format!("Nao foi possivel abrir o navegador automaticamente. URL:\n\n{url}\n\nErro: {error}"),
        );
    }
}
