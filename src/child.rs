use artisan_middleware::dusa_collection_utils::errors::Errors;
use artisan_middleware::dusa_collection_utils::log;
use artisan_middleware::process_manager::{
    spawn_complex_process, spawn_simple_process, SupervisedChild,
};
use artisan_middleware::state_persistence::{log_error, update_state, wind_down_state};
use artisan_middleware::{
    dusa_collection_utils::{errors::ErrorArrayItem, log::LogLevel, types::PathType},
    state_persistence::AppState,
};
// use std::{env, fs};
use std::fs;
use tokio::process::Command;

use crate::config::AppSpecificConfig;

pub async fn create_child(
    mut state: &mut AppState,
    state_path: &PathType,
    settings: &AppSpecificConfig,
) -> SupervisedChild {
    log!(LogLevel::Trace, "Creating child process...");

    let mut command: Command = Command::new("npm");

    command
        .args(&["--prefix", &settings.clone().project_path, "run", "dev"]) // Updated to run "build" instead of "start"
        // .env("NODE_ENV", "production") // Set NODE_ENV=production
        .env("PORT", "3001"); // Set PORT=3000

    // using capture output false to pipe to stdout to the running stdout
    match spawn_complex_process(&mut command, Some(settings.project_path()), false, false).await {
        Ok(mut spawned_child) => {
            // initialize monitor loop.
            spawned_child.monitor_usage().await;
            // read the pid from the state
            let pid: u32 = match spawned_child.get_pid().await {
                Ok(xid) => xid,
                Err(_) => {
                    let error_item = ErrorArrayItem::new(
                        Errors::InputOutput,
                        "No pid for supervised child".to_owned(),
                    );
                    log_error(state, error_item, &state_path).await;
                    wind_down_state(state, &state_path).await;
                    std::process::exit(100);
                }
            };

            // save the pid somewhere
            let pid_file: PathType =
                PathType::Content(format!("/tmp/.{}_pg.pid", state.config.app_name));

            if let Err(error) = fs::write(pid_file, pid.to_string()) {
                let error_ref = error.get_ref().unwrap_or_else(|| {
                    log!(LogLevel::Trace, "{:?}", error);
                    std::process::exit(100);
                });

                let error_item = ErrorArrayItem::new(Errors::InputOutput, error_ref.to_string());
                log_error(&mut state, error_item, &state_path).await;
                wind_down_state(&mut state, &state_path).await;
                std::process::exit(100);
            }
            log!(LogLevel::Info, "Child process spawned, pid info saved");

            if let Ok(metrics) = spawned_child.get_metrics().await {
                update_state(&mut state, &state_path, Some(metrics)).await;
            }
            return spawned_child;
        }
        Err(error) => {
            log_error(&mut state, error, &state_path).await;
            wind_down_state(&mut state, &state_path).await;
            std::process::exit(100);
        }
    }
}

// Sometimes we need a lil npm install
pub async fn run_install_process(
    settings: &AppSpecificConfig,
    state: &mut AppState,
    state_path: &PathType,
) -> Result<(), ErrorArrayItem> {
    let mut process = spawn_simple_process(
        Command::new("npm")
            .arg("--prefix")
            .arg(settings.clone().project_path)
            .arg("install"),
        // .env("NODE_ENV", "production"),
        false,
        state,
        state_path,
    )
    .await
    .map_err(ErrorArrayItem::from)?;

    match process.wait().await {
        Ok(_) => return Ok(()),
        Err(err) => {
            return Err(ErrorArrayItem::new(Errors::GeneralError, err.to_string()));
        }
    }
}
