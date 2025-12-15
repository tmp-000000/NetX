use chrono::{DateTime, Local};
use std::sync::{Arc, Mutex};

use netx::config::{Config, ConfigManager};

#[derive(Clone, Debug)]
pub struct LogLine {
    pub time: DateTime<Local>,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum RealStatus {
    Disconnected,
    Connecting { profile: String },
    Connected { profile: String, pid: u32 },
    Error { message: String },
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<Config>>,
    pub status: Arc<Mutex<RealStatus>>,
    pub logs: Arc<Mutex<Vec<LogLine>>>,
}

impl AppState {
    pub fn new() -> Self {
        let config = ConfigManager::load_config().unwrap_or_default();

        Self {
            config: Arc::new(Mutex::new(config)),
            status: Arc::new(Mutex::new(RealStatus::Disconnected)),
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Получаем актуальные профили
    pub fn profiles(&self) -> Vec<netx::config::Profile> {
        netx::config::ConfigManager::load_config()
            .map(|c| c.profiles)
            .unwrap_or_default()
    }

    // Удаляем профиль по индексу
    pub fn delete_profile(&self, index: usize) {
        netx::config::ConfigManager::delete_profile(index);
        self.push_log(format!("Profile at index {} deleted", index));
    }

    pub fn push_log<S: Into<String>>(&self, msg: S) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(LogLine {
            time: Local::now(),
            message: msg.into(),
        });

        let len = logs.len();
        if len > 200 {
            logs.drain(0..len - 200);
        }
    }
}
