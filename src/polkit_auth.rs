use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};

/// Installed by packaging (see polkit/); the polkit action ties auth behavior
/// to this exact helper path so `pkexec` only ever prompts for this narrow helper.
const HELPER_PATH: &str = "/usr/lib/openfortigui-gtk/openfortigui-privileged-helper";

fn pkexec_helper(args: &[&str]) -> Command {
    let mut command = Command::new("pkexec");
    command.arg(HELPER_PATH).args(args);
    command
}

pub fn start_vpn_privileged(config_path: &Path) -> io::Result<Child> {
    pkexec_helper(&["start", &config_path.to_string_lossy()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

pub fn stop_vpn_privileged(pid: u32) -> io::Result<Child> {
    pkexec_helper(&["stop", &pid.to_string()]).spawn()
}
