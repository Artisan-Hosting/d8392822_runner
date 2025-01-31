# Child Process Manager with One-Shot Process Execution

This application is designed to manage child processes in a controlled environment, providing resource management, persistent state, and monitoring functionality. It also includes support for executing a one-shot command (e.g., `npm install`) before creating the main child process. This template can be used for other applications that require process spawning, monitoring, and management in an async Rust environment using `tokio`.

## Features

- **One-Shot Process Execution**: Execute a one-shot command (such as setup or installation commands) before spawning a long-running child process.
- **Child Process Group Management**: Create and control child processes, including the ability to restart processes if terminated.
- **Directory Monitoring**: Monitor a specific directory for changes and trigger actions based on a threshold of change events.
- **State Persistence**: Application state is saved to disk and reloaded on startup, allowing for resilience after unexpected shutdowns.
- **Asynchronous Execution**: Built using `tokio` for non-blocking async support.
- **Error Logging**: Detailed error logging to assist in debugging and monitoring.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Dependencies](#dependencies)
3. [Usage](#usage)
4. [Configuration](#configuration)
5. [Customization](#customization)
6. [Contributing](#contributing)
7. [License](#license)

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable version recommended)
- [Tokio](https://tokio.rs) for asynchronous execution
- A UNIX-based system for process management (e.g., Linux or macOS)

### Installation

1. **Clone the repository**:

    ```sh
    git clone https://github.com/yourusername/child-process-manager.git
    cd child-process-manager
    ```

2. **Install dependencies**:

    Dependencies are managed by Cargo and will be automatically installed when you build the project.

3. **Build the project**:

    ```sh
    cargo build --release
    ```

4. **Run the project**:

    ```sh
    cargo run
    ```

## Usage

### Running the Application

This application initializes its state, loads configuration settings, and then runs a one-shot process (e.g., `npm install`) before creating a child process. It monitors a directory for changes and restarts the child process if needed.

### Main Functionality Overview

The `main` function of the application follows these key steps:

1. **Initialization**:
   - Load the main configuration using `get_config()`.
   - Load or initialize the application state using `StatePersistence::load_state()`.
   - Set up logging and other initial settings.

2. **Run One-Shot Command**:
   - The one-shot process (`run_one_shot_process()`) is executed asynchronously before creating the main child process. This step can be used for any required setup, such as running `npm install`.

3. **Spawn Child Process**:
   - A child process is created using `create_child()`. The process information is logged, and state is updated accordingly.

4. **Directory Monitoring**:
   - The `monitor_directory()` function is used to monitor the specified directory for changes.
   - When the configured number of changes (`changes_needed`) is reached, the child process is restarted.

5. **Main Event Loop**:
   - The main loop uses `tokio::select!` to wait for directory change events or periodically check the status of the child process.
   - If the monitored directory changes enough times, the child process is terminated and restarted.
   - The periodic task checks the status of the child process and restarts it if it is not running.

## Configuration

### Configuration File

The application uses two levels of configuration: the general configuration (`AppConfig`) and the specific configuration (`AppSpecificConfig`). These configurations are loaded at startup and control the behavior of the application.

#### `AppConfig`

The `AppConfig` is loaded using `get_config()`, which provides the main configuration options such as:

- **`app_name`**: The name of the application (defaults to the name in `Cargo.toml`).
- **`version`**: The version of the application (defaults to the version in `Cargo.toml`).
- **`debug_mode`**: Enables or disables debug-level logging.
- **`log_level`**: Sets the logging level (`Trace`, `Info`, `Error`, etc.).
- **`database`, `aggregator`, `git`**: Optional settings that can be configured as needed, defaulting to `None`.

#### `AppSpecificConfig`

The `AppSpecificConfig` provides application-specific settings and is loaded using the `specific_config()` function. It includes:

- **`interval_seconds`**: The interval for periodic checks, in seconds.
- **`monitor_path`**: The directory path to monitor for changes.
- **`project_path`**: The path to the project that needs one-shot processing or monitoring.
- **`changes_needed`**: The number of changes needed in the monitored directory to trigger a restart of the child process.

These configurations are loaded from a file called `Config.toml`, which can be customized to match your environment.

### Logging

The application has a built-in logging system using the `log!()` macro. You can adjust the log level via the configuration file or within the code by calling `set_log_level()`. Different log levels are used throughout the code to provide varying levels of detail (`Trace`, `Info`, `Debug`, `Error`).

### State Persistence

The state of the application (`AppState`) is managed through the `StatePersistence` module and saved to a file to ensure resilience. The state includes information like:

- **`data`**: General state information.
- **`last_updated`**: Timestamp of the last update.
- **`event_counter`**: Count of events handled.
- **`is_active`**: Indicates if the application is currently active.
- **`error_log`**: Logs of any errors that have occurred.

The state is saved using `StatePersistence::save_state()` and reloaded on startup, allowing the application to recover from unexpected shutdowns.

## Customization

This application is configured with a specific runtime in mind, but it is meant to serve as a template that can be adapted to other use cases. To customize it for different scenarios:

1. **Modify One-Shot Process**:
   - The `run_one_shot_process()` function will need to be adapted based on the setup or initialization requirements of your use case. For example, you could replace `npm install` with any command needed before starting the main process.

    ```rust
    async fn run_one_shot_process() -> Result<(), String> {
        let output = Command::new("your_command_here")
            .arg("your_arguments_here")
            .output()
            .await
            .map_err(|err| format!("Failed to execute command: {}", err))?;
    
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Command failed: {}", stderr));
        }

        Ok(())
    }
    ```

2. **Modify Child Process Creation**:
   - The `create_child()` function should be customized based on the process that you want to manage. This may involve setting different command arguments, working directories, or environment variables based on your application's needs.

3. **Adapt Directory Monitoring**:
   - Update the `monitor_directory()` function to monitor different paths or handle events in a way that is specific to your application's requirements.

4. **Configuration Changes**:
   - Modify the `Config.toml` file to include your application-specific settings. The structure of `AppConfig` and `AppSpecificConfig` can be extended to meet the specific configuration needs of your application.

## Example Use Cases

This template can be used as a base for projects that require controlled process management:

- **Web Server Managers**: Run setup commands (like `npm install` or `pip install`) before starting a web server.
- **Containerized Applications**: Configure and run initialization steps before launching containers or services.
- **Task Automation**: Automate setup and cleanup tasks before running a worker process.

## Contributing

Contributions are welcome! If you'd like to contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch (`git checkout -b feature/your-feature`).
3. Make your changes.
4. Commit your changes (`git commit -m 'Add some feature'`).
5. Push to the branch (`git push origin feature/your-feature`).
6. Open a Pull Request.

## License

This project is licensed under the AHSLv1. See the [License](License) file for details.
