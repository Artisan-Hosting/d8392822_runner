use artisan_middleware::{
    aggregator::Status, common::update_state, config::AppConfig, dusa_collection_utils::{
        self,
        stringy::Stringy,
        version::{SoftwareVersion, Version, VersionCode},
    }, state_persistence::{AppState, StatePersistence}, timestamp::current_timestamp, version::{aml_version, str_to_version}
};
use colored::Colorize;
use config::{Config, ConfigError, File};
use dusa_collection_utils::{
    log,
    log::{LogLevel, set_log_level},
    types::PathType,
};
use serde::Deserialize;
use std::fmt;

pub fn get_config() -> AppConfig {
    let mut config: AppConfig = match AppConfig::new() {
        Ok(loaded_data) => loaded_data,
        Err(e) => {
            log!(LogLevel::Error, "Couldn't load config: {}", e.to_string());
            std::process::exit(100)
        }
    };
    config.app_name = Stringy::from(env!("CARGO_PKG_NAME").to_string());

    let raw_version: SoftwareVersion = {
        // defining the version
        let library_version: Version = aml_version();
        let software_version: Version =
            str_to_version(env!("CARGO_PKG_VERSION"), Some(VersionCode::Production));

        SoftwareVersion {
            application: software_version,
            library: library_version,
        }
    };

    config.version = match serde_json::to_string(&raw_version) {
        Ok(ver) => ver,
        Err(err) => {
            log!(LogLevel::Error, "{}", err);
            std::process::exit(100);
        }
    };

    config.database = None;
    config
}

pub async fn generate_application_state(state_path: &PathType, config: &AppConfig) -> AppState {
    match StatePersistence::load_state(&state_path).await {
        Ok(mut loaded_data) => {
            log!(LogLevel::Info, "Loaded previous state data");
            log!(LogLevel::Trace, "Previous state data: {:#?}", loaded_data);
            loaded_data.data = String::from("Initializing");
            loaded_data.config.debug_mode = config.debug_mode;
            loaded_data.last_updated = current_timestamp();
            loaded_data.config.log_level = config.log_level;
            loaded_data.status = Status::Starting;
            loaded_data.pid = std::process::id();
            set_log_level(loaded_data.config.log_level);
            loaded_data.error_log.clear();
            update_state(&mut loaded_data, &state_path, None).await;
            loaded_data
        }
        Err(e) => {
            log!(LogLevel::Warn, "No previous state loaded, creating new one");
            log!(LogLevel::Debug, "Error loading previous state: {}", e);
            let mut state = AppState {
                data: String::new(),
                last_updated: current_timestamp(),
                event_counter: 0,
                error_log: vec![],
                config: config.clone(),
                name: config.app_name.to_string(),
                pid: std::process::id(),
                version: {
                    // defining the version
                    let library_version: Version = aml_version();
                    let software_version: Version =
                        str_to_version(env!("CARGO_PKG_VERSION"), Some(VersionCode::Production));
            
                    SoftwareVersion {
                        application: software_version,
                        library: library_version,
                    }
                },
                system_application: false,
                status: Status::Starting,
            };
            state.data = String::from("Initializing");
            state.config.debug_mode = config.debug_mode;
            state.last_updated = current_timestamp();
            state.config.log_level = config.log_level;
            set_log_level(state.config.log_level);
            state.error_log.clear();
            update_state(&mut state, &state_path, None).await;

            state
        }
    }
}


pub fn specific_config() -> Result<AppSpecificConfig, ConfigError> {
    let mut builder = Config::builder();
    builder = builder.add_source(File::with_name("Config").required(false));

    let settings = builder.build()?;
    let app_specific: AppSpecificConfig = settings.get("app_specific")?;

    Ok(app_specific)
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppSpecificConfig {
    pub interval_seconds: u32,
    pub monitor_path: String,
    pub project_path: String,
    pub changes_needed: i32,
}

#[allow(dead_code)]
impl AppSpecificConfig {
    pub fn safe_path(&self) -> PathType {
        let self_cloned = self.clone();
        let path = PathType::Content(self_cloned.monitor_path);
        if !path.exists() {
            log!(LogLevel::Error, "The path {} doesn't exist", path);
            std::process::exit(0)
        } else {
            match path.canonicalize() {
                Ok(canon_path) => PathType::PathBuf(canon_path),
                Err(e) => {
                    log!(
                        LogLevel::Error,
                        "Failed to canonicalize path: {}, using default: {}",
                        e,
                        path
                    );
                    path
                }
            }
        }
    }
    pub fn project_path(&self) -> PathType {
        let self_cloned = self.clone();
        let path = PathType::Content(self_cloned.project_path);
        if !path.exists() {
            log!(LogLevel::Error, "The path {} doesn't exist", path);
            std::process::exit(0)
        } else {
            match path.canonicalize() {
                Ok(canon_path) => PathType::PathBuf(canon_path),
                Err(e) => {
                    log!(
                        LogLevel::Error,
                        "Failed to canonicalize path: {}, using default: {}",
                        e,
                        path
                    );
                    path
                }
            }
        }
    }
}

impl fmt::Display for AppSpecificConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {{\n\
             \t{}: {},\n\
             \t{}: {},\n\
             \t{}: {},\n\
             \t{}: {},\n\
             }}",
            "AppSpecificConfig".cyan().bold(),
            "interval_seconds".yellow(),
            self.interval_seconds.to_string().green(),
            "monitor_path".yellow(),
            self.monitor_path.clone().green(),
            "project_path".yellow(),
            self.project_path.clone().green(),
            "changes_needed".yellow(),
            self.changes_needed.to_string().green()
        )
    }
}
