use std::sync::Arc;

use netx::config::{Config, ConfigManager};

fn main() -> anyhow::Result<()> {
    let config: Config = ConfigManager::load_config()?;
    platform::run_tray(Arc::new(config))
}

mod platform {
    use netx::config::Config;
    use netx::tray;
    use std::sync::Arc;

    #[cfg(windows)]
    pub fn run_tray(config: Arc<Config>) -> anyhow::Result<()> {
        tray::run(config)
    }

    #[cfg(target_os = "linux")]
    pub fn run_tray(config: Arc<Config>) -> anyhow::Result<()> {
        tray::run(config)
    }
}
