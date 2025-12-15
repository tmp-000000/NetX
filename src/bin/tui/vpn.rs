use crate::state::AppState;
use crate::state::RealStatus;
use std::process::Command;

pub fn start(profile: String, state: AppState) {
    {
        let mut s = state.status.lock().unwrap();
        *s = RealStatus::Connecting {
            profile: profile.clone(),
        };
    }
    state.push_log(format!("starting sing-box for {}", profile));

    #[cfg(target_os = "windows")]
    let _ = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Start-Process -FilePath \"{}\" -ArgumentList \"--config config\\config.json\"",
                profile
            ),
        ])
        .spawn();

    #[cfg(target_os = "linux")]
    let _ = Command::new("sudo")
        .args(["systemctl", "restart", "netx-singbox.service"])
        .status();
}

pub fn stop(state: AppState) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("powershell")
            .args(["-Command", "Stop-Process -Name sing-box"])
            .status();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("sudo")
            .args(["systemctl", "stop", "netx-singbox.service"])
            .status();
    }

    {
        let mut s = state.status.lock().unwrap();
        *s = RealStatus::Disconnected;
    }
    state.push_log("sing-box stopped");
    Ok(())
}
