use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io::Stdout;
use std::time::Duration;

use crate::state::{AppState, RealStatus};
use crate::vpn;
use netx::config::Profile;

pub fn run(term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let state = AppState::new();
    let mut selected: usize = 0;

    loop {
        // vpn::sync(&state);

        let profiles = state.profiles();

        term.draw(|f| ui(f, &state, &profiles, selected))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        if !profiles.is_empty() {
                            selected = (selected + 1) % profiles.len();
                        }
                    }
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(profile) = profiles.get(selected) {
                            vpn::start(profile.name.clone(), state.clone());
                        }
                    }
                    KeyCode::Char('x') => {
                        let _ = vpn::stop(state.clone());
                    }
                    KeyCode::Char('d') => {
                        if selected < profiles.len() {
                            state.delete_profile(selected);
                            if selected >= profiles.len() && selected > 0 {
                                selected -= 1;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, state: &AppState, profiles: &[Profile], selected: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // статус
            Constraint::Min(10),   // список профилей + детали
            Constraint::Length(5), // логи
        ])
        .split(f.area());

    // --- Статус VPN ---
    let status_line = match state.status.lock().unwrap().clone() {
        RealStatus::Disconnected => "🔴 Disconnected".to_string(),
        RealStatus::Connecting { profile } => format!("🟡 Connecting to {}", profile),
        RealStatus::Connected { profile, pid } => {
            format!("🟢 Connected to {} (pid {})", profile, pid)
        }
        RealStatus::Error { message } => format!("❌ {}", message),
    };

    f.render_widget(
        Paragraph::new(status_line).block(Block::default().borders(Borders::ALL).title("Status")),
        chunks[0],
    );

    // --- Профили + детали ---
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // Список профилей
    let items: Vec<ListItem> = profiles
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let style = if i == selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(p.name.clone()).style(style)
        })
        .collect();

    f.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title("Profiles")),
        main_chunks[0],
    );

    // Детали выбранного профиля
    let detail_text = if let Some(profile) = profiles.get(selected) {
        format!(
            "Name: {}\nType: {:?}\nServer: {}:{}\nTLS Server Name: {}\nUTLS Fingerprint: {}\nReality PK: {}\nTransport: {}",
            profile.name,
            "VLESS",
            profile.server,
            profile.server_port,
            profile.tls.server_name,
            profile.tls.utls.fingerprint,
            profile.tls.reality.public_key,
            profile.tls.transport.service_name,
        )
    } else {
        "No profile selected".to_string()
    };

    f.render_widget(
        Paragraph::new(detail_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Profile Details"),
        ),
        main_chunks[1],
    );

    // --- Логи ---
    let logs = state.logs.lock().unwrap();
    let log_text = logs
        .iter()
        .rev()
        .take(5)
        .map(|l| format!("{} {}", l.time.format("%H:%M:%S"), l.message))
        .collect::<Vec<_>>()
        .join("\n");

    f.render_widget(
        Paragraph::new(log_text).block(Block::default().borders(Borders::ALL).title("Logs")),
        chunks[2],
    );
}
