pub mod daily_reporter;
pub mod event_reactor;
pub mod log_watcher;
pub mod resource_analyzer;
pub mod self_healer;

use crate::config::Config;
use crate::notifier::Notifier;
use bollard::Docker;
use bollard::system::EventsOptions;
use futures_util::StreamExt;
use futures_util::future::join_all;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, watch};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep, timeout};

type TaskHandles = Arc<Mutex<Vec<JoinHandle<()>>>>;
type ActiveLogWatchers = Arc<Mutex<HashSet<String>>>;

#[derive(Debug)]
enum SessionOutcome {
    Shutdown,
    Disconnected { saw_event: bool },
}

pub struct Monitor {
    config: Config,
    notifier: Notifier,
    log_alert_cache: log_watcher::LogAlertCache,
    restart_tracker: self_healer::RestartTracker,
    daily_stats: daily_reporter::SharedDailyStats,
}

impl Monitor {
    pub fn new(config: Config) -> Self {
        let notifier = Notifier::new(config.clone());
        let log_alert_cache = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let restart_tracker = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let daily_stats = daily_reporter::new_shared_daily_stats();
        Self {
            config,
            notifier,
            log_alert_cache,
            restart_tracker,
            daily_stats,
        }
    }

    pub async fn run(&self) -> Result<(), String> {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let signal_tx = shutdown_tx.clone();
        let signal_handle = tokio::spawn(async move {
            let signal_name = shutdown_signal().await;
            println!("\nDockture: Shutdown signal ({}) received.", signal_name);
            let _ = signal_tx.send(true);
        });

        let mut reconnect_delay = Duration::from_secs(1);

        while !*shutdown_rx.borrow() {
            let docker = match connect_docker_session().await {
                Ok(docker) => {
                    println!("Successfully connected to Docker. Starting monitor session...");
                    docker
                }
                Err(e) => {
                    eprintln!("Docker connection failed: {}", e);
                    if wait_or_shutdown(shutdown_rx.clone(), reconnect_delay).await {
                        break;
                    }
                    reconnect_delay = next_reconnect_delay(reconnect_delay);
                    continue;
                }
            };

            match self.run_monitor_session(docker, shutdown_rx.clone()).await {
                SessionOutcome::Shutdown => break,
                SessionOutcome::Disconnected { saw_event } => {
                    if saw_event {
                        reconnect_delay = Duration::from_secs(1);
                    }
                    eprintln!(
                        "Dockture: Docker monitor session ended; reconnecting in {}s.",
                        reconnect_delay.as_secs()
                    );
                    if wait_or_shutdown(shutdown_rx.clone(), reconnect_delay).await {
                        break;
                    }
                    reconnect_delay = next_reconnect_delay(reconnect_delay);
                }
            }
        }

        let _ = shutdown_tx.send(true);
        signal_handle.abort();
        let _ = signal_handle.await;
        let guard = self.daily_stats.lock().await;
        daily_reporter::save_daily_stats_to_file(&guard);
        println!("Dockture: Shutdown complete.");

        Ok(())
    }

    async fn run_monitor_session(
        &self,
        docker: Docker,
        mut shutdown_rx: watch::Receiver<bool>,
    ) -> SessionOutcome {
        let (session_tx, session_rx) = watch::channel(false);
        let task_handles: TaskHandles = Arc::new(Mutex::new(Vec::new()));
        let active_log_watchers: ActiveLogWatchers = Arc::new(Mutex::new(HashSet::new()));

        spawn_resource_monitor(
            docker.clone(),
            self.config.clone(),
            self.notifier.clone(),
            self.daily_stats.clone(),
            session_rx.clone(),
            &task_handles,
        )
        .await;

        spawn_daily_reporter(
            docker.clone(),
            self.config.clone(),
            self.notifier.clone(),
            self.daily_stats.clone(),
            session_rx.clone(),
            &task_handles,
        )
        .await;

        if let Err(e) = self
            .reconcile_running_containers(
                &docker,
                session_rx.clone(),
                &task_handles,
                &active_log_watchers,
            )
            .await
        {
            eprintln!("Dockture: failed to reconcile running containers: {}", e);
        }

        let mut filters = HashMap::new();
        filters.insert("type".to_string(), vec!["container".to_string()]);
        filters.insert(
            "event".to_string(),
            vec![
                "die".to_string(),
                "oom".to_string(),
                "health_status".to_string(),
                "start".to_string(),
            ],
        );

        let options = EventsOptions {
            since: None,
            until: None,
            filters,
        };

        let mut events_stream = docker.events(Some(options));
        let mut saw_event = false;

        let outcome = loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        break SessionOutcome::Shutdown;
                    }
                }
                event_res = events_stream.next() => {
                    match event_res {
                        Some(Ok(event)) => {
                            saw_event = true;
                            let event_ctx = event_reactor::EventContext {
                                docker: &docker,
                                config: &self.config,
                                notifier: &self.notifier,
                                log_alert_cache: &self.log_alert_cache,
                                restart_tracker: &self.restart_tracker,
                                daily_stats: &self.daily_stats,
                                shutdown: &session_rx,
                                task_handles: &task_handles,
                                active_log_watchers: &active_log_watchers,
                            };
                            if let Err(e) = event_reactor::handle_docker_event(event_ctx, event)
                                .await
                            {
                                eprintln!("Error handling event: {}", e);
                            }
                        }
                        Some(Err(e)) => {
                            eprintln!("Error reading Docker event stream: {}", e);
                            break SessionOutcome::Disconnected { saw_event };
                        }
                        None => {
                            eprintln!("Dockture: Docker event stream ended.");
                            break SessionOutcome::Disconnected { saw_event };
                        }
                    }
                }
            }
        };

        let _ = session_tx.send(true);
        {
            let guard = self.daily_stats.lock().await;
            daily_reporter::save_daily_stats_to_file(&guard);
        }
        println!("Dockture: Waiting for monitor session tasks to stop...");
        join_child_tasks(&task_handles).await;
        let guard = self.daily_stats.lock().await;
        daily_reporter::save_daily_stats_to_file(&guard);
        println!("Dockture: Monitor session tasks stopped.");

        outcome
    }

    async fn reconcile_running_containers(
        &self,
        docker: &Docker,
        shutdown: watch::Receiver<bool>,
        task_handles: &TaskHandles,
        active_log_watchers: &ActiveLogWatchers,
    ) -> Result<(), String> {
        let list_options = Some(bollard::container::ListContainersOptions::<String> {
            all: false,
            ..Default::default()
        });
        let active_containers = docker
            .list_containers(list_options)
            .await
            .map_err(|e| format!("Failed to list running containers: {}", e))?;

        for c in active_containers {
            let Some(id) = c.id else {
                continue;
            };
            let name = c
                .names
                .as_ref()
                .and_then(|names| names.first())
                .map(|n| n.trim_start_matches('/'))
                .unwrap_or("unknown");

            if name == "unknown" || !self.config.is_container_monitored(name) {
                continue;
            }

            let spec = log_watcher::LogWatcherSpec {
                docker: docker.clone(),
                container_id: id,
                container_name: name.to_string(),
                config: self.config.clone(),
                notifier: self.notifier.clone(),
                cache: self.log_alert_cache.clone(),
                daily_stats: self.daily_stats.clone(),
                shutdown: shutdown.clone(),
            };

            spawn_log_watcher(spec, task_handles, active_log_watchers).await;
        }

        Ok(())
    }
}

async fn connect_docker_session() -> Result<Docker, String> {
    println!("Connecting to Docker daemon (socket / DOCKER_HOST)...");
    let docker = crate::utils::connect_docker()?;
    docker
        .ping()
        .await
        .map_err(|e| format!("Failed to ping Docker daemon (is it running?): {}", e))?;
    Ok(docker)
}

async fn spawn_resource_monitor(
    docker: Docker,
    config: Config,
    notifier: Notifier,
    daily_stats: daily_reporter::SharedDailyStats,
    shutdown: watch::Receiver<bool>,
    task_handles: &TaskHandles,
) {
    let handle = tokio::spawn(async move {
        if let Err(e) =
            resource_analyzer::run_resource_monitor(docker, config, notifier, daily_stats, shutdown)
                .await
        {
            eprintln!("Resource monitor error: {}", e);
        }
    });
    task_handles.lock().await.push(handle);
}

async fn spawn_daily_reporter(
    docker: Docker,
    config: Config,
    notifier: Notifier,
    daily_stats: daily_reporter::SharedDailyStats,
    shutdown: watch::Receiver<bool>,
    task_handles: &TaskHandles,
) {
    let handle = tokio::spawn(async move {
        daily_reporter::run_daily_reporter(docker, config, notifier, daily_stats, shutdown).await;
    });
    task_handles.lock().await.push(handle);
}

pub(super) async fn spawn_log_watcher(
    spec: log_watcher::LogWatcherSpec,
    task_handles: &TaskHandles,
    active_log_watchers: &ActiveLogWatchers,
) {
    {
        let mut active = active_log_watchers.lock().await;
        if !active.insert(spec.container_id.clone()) {
            return;
        }
    }

    let active_log_watchers_clone = active_log_watchers.clone();
    let registry_id = spec.container_id.clone();
    let handle = tokio::spawn(async move {
        let res = log_watcher::monitor_container_logs(spec).await;
        if let Err(e) = res {
            eprintln!("Log monitor error: {}", e);
        }
        active_log_watchers_clone.lock().await.remove(&registry_id);
    });
    task_handles.lock().await.push(handle);
}

async fn wait_or_shutdown(mut shutdown: watch::Receiver<bool>, delay: Duration) -> bool {
    tokio::select! {
        _ = shutdown.changed() => *shutdown.borrow(),
        _ = sleep(delay) => false,
    }
}

fn next_reconnect_delay(current: Duration) -> Duration {
    std::cmp::min(current * 2, Duration::from_secs(60))
}

async fn join_child_tasks(task_handles: &Arc<Mutex<Vec<JoinHandle<()>>>>) {
    let mut handles = {
        let mut guard = task_handles.lock().await;
        guard.drain(..).collect::<Vec<_>>()
    };

    match timeout(Duration::from_secs(10), join_all(handles.iter_mut())).await {
        Ok(results) => {
            for result in results {
                if let Err(e) = result {
                    eprintln!("Dockture: monitor task failed during shutdown: {}", e);
                }
            }
        }
        Err(_) => {
            eprintln!(
                "Dockture: timed out while waiting for monitor tasks; aborting remaining tasks."
            );
            for handle in &handles {
                if !handle.is_finished() {
                    handle.abort();
                }
            }
            for result in join_all(handles).await {
                if let Err(e) = result {
                    eprintln!(
                        "Dockture: monitor task aborted after shutdown timeout: {}",
                        e
                    );
                }
            }
        }
    }
}

#[cfg(unix)]
async fn shutdown_signal() -> &'static str {
    let mut sigterm = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    {
        Ok(signal) => signal,
        Err(e) => {
            eprintln!(
                "Dockture: failed to install SIGTERM handler ({}); waiting for SIGINT only.",
                e
            );
            let _ = tokio::signal::ctrl_c().await;
            return "SIGINT";
        }
    };

    tokio::select! {
        _ = tokio::signal::ctrl_c() => "SIGINT",
        _ = sigterm.recv() => "SIGTERM",
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() -> &'static str {
    let _ = tokio::signal::ctrl_c().await;
    "SIGINT"
}
