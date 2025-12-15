use crate::config::Config;
use std::process::Command;
use std::sync::Arc;
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::MenuEvent,
    menu::{Menu, MenuItem},
};

fn load_icon() -> Icon {
    let width = 16;
    let height = 16;
    let mut rgba = Vec::new();
    for _ in 0..width * height {
        rgba.push(0); // R
        rgba.push(0); // G
        rgba.push(255); // B
        rgba.push(255); // A
    }
    Icon::from_rgba(rgba, width as u32, height as u32).expect("Invalid icon")
}

// Linux реализация с GTK event loop
#[cfg(target_os = "linux")]
pub fn run(config: Arc<Config>) -> anyhow::Result<()> {
    gtk::init().expect("GTK init failed");

    let menu = Menu::new();
    let open_tui = MenuItem::new("Open TUI", true, None);
    let quit = MenuItem::new("Quit", true, None);

    let open_id = open_tui.id().clone();
    let quit_id = quit.id().clone();

    menu.append(&open_tui)?;
    menu.append(&quit)?;

    let _tray = TrayIconBuilder::new()
        .with_icon(load_icon())
        .with_menu(Box::new(menu))
        .with_tooltip("NetX")
        .build()?;

    let rx = MenuEvent::receiver();

    // GTK event loop для Linux
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if let Ok(event) = rx.try_recv() {
            if event.id == open_id {
                open_tui_action(&config);
            } else if event.id == quit_id {
                gtk::main_quit();
            }
        }
        glib::ControlFlow::Continue
    });

    gtk::main();
    Ok(())
}

// Windows реализация с нативным event loop
#[cfg(windows)]
pub fn run(config: Arc<Config>) -> anyhow::Result<()> {
    let menu = Menu::new();
    let open_tui = MenuItem::new("Open TUI", true, None);
    let quit = MenuItem::new("Quit", true, None);

    let open_id = open_tui.id();
    let quit_id = quit.id();

    menu.append(&open_tui)?;
    menu.append(&quit)?;

    let _tray = TrayIconBuilder::new()
        .with_icon(load_icon())
        .with_menu(Box::new(menu))
        .with_tooltip("NetX")
        .build()?;

    let rx = MenuEvent::receiver();

    // Простой blocking loop для Windows
    loop {
        if let Ok(event) = rx.recv() {
            if event.id == open_id {
                open_tui_action(&config);
            } else if event.id == quit_id {
                std::process::exit(0);
            }
        }
    }
}

// macOS реализация
#[cfg(target_os = "macos")]
pub fn run(config: Arc<Config>) -> anyhow::Result<()> {
    let menu = Menu::new();
    let open_tui = MenuItem::new("Open TUI", true, None);
    let quit = MenuItem::new("Quit", true, None);

    let open_id = open_tui.id();
    let quit_id = quit.id();

    menu.append(&open_tui)?;
    menu.append(&quit)?;

    let _tray = TrayIconBuilder::new()
        .with_icon(load_icon())
        .with_menu(Box::new(menu))
        .with_tooltip("NetX")
        .build()?;

    let rx = MenuEvent::receiver();

    // Простой blocking loop для macOS
    loop {
        if let Ok(event) = rx.recv() {
            if event.id == open_id {
                open_tui_action(&config);
            } else if event.id == quit_id {
                std::process::exit(0);
            }
        }
    }
}

fn open_tui_action(config: &Config) {
    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/c", "start", &config.terminal.clone(), "/k", "./tui"])
            .spawn()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        Command::new(config.terminal.clone())
            .args(["-e", "./tui"])
            .spawn()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-a", config.terminal.clone(), "./tui"])
            .spawn()
            .ok();
    }
}
