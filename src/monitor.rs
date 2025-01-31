use artisan_middleware::dusa_collection_utils::log;
use artisan_middleware::dusa_collection_utils::log::LogLevel;
use artisan_middleware::dusa_collection_utils::rwarc::LockWithTimeout;
use artisan_middleware::dusa_collection_utils::types::PathType;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

pub async fn monitor_directory(dir: PathType) -> notify::Result<UnboundedReceiver<Event>> {
    log!(
        LogLevel::Trace,
        "Initializing directory watcher for path: {}",
        dir
    );

    let (watcher_tx, watcher_rx) = channel();
    let (event_tx, event_rx) = unbounded_channel();

    // Wrap the watcher in an Arc<Mutex<>> to manage its lifetime
    let watcher = LockWithTimeout::new(RecommendedWatcher::new(watcher_tx, Config::default())?);

    // Start watching the directory
    if let Ok(mut watcher) = watcher.try_write().await {
        watcher.watch(&dir, RecursiveMode::Recursive)?;
    } else {
        log!(LogLevel::Error, "Never started watching directory");
    };

    log!(LogLevel::Trace, "Started watching directory: {}", dir);

    // Clone the Arc to move into the thread
    let watcher_clone = watcher.clone();

    // Spawn a thread to forward events to the async channel
    log!(
        LogLevel::Trace,
        "Spawning thread to handle directory events..."
    );
    thread::spawn(move || {
        log!(LogLevel::Trace, "Directory event handler thread started.");

        loop {
            match watcher_rx.recv() {
                Ok(event) => match event {
                    Ok(event) => {
                        log!(
                            LogLevel::Trace,
                            "Directory change event received: {:#?}",
                            event
                        );
                        if event_tx.send(event).is_err() {
                            log!(
                                LogLevel::Error,
                                "Failed to send event: Event channel closed."
                            );
                            break;
                        } else {
                            log!(
                                LogLevel::Trace,
                                "Event successfully forwarded to async channel."
                            );
                        }
                    }
                    Err(e) => {
                        log!(
                            LogLevel::Error,
                            "Error receiving event from watcher: {:?}",
                            e
                        );
                    }
                },
                Err(recv_err) => {
                    log!(
                        LogLevel::Error,
                        "Error receiving from watcher channel: {}",
                        recv_err
                    );
                    // Optional: add a small delay to prevent a busy loop if an error keeps occurring
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }

        // Drop the watcher explicitly when done
        drop(watcher_clone);

        log!(LogLevel::Trace, "Directory event handler thread exiting.");
    });

    log!(LogLevel::Trace, "Returning event receiver to caller.");
    Ok(event_rx)
}
