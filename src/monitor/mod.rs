pub mod event_reactor;
pub mod log_watcher;
pub mod resource_analyzer;
pub mod self_healer;

use crate::config::Config;
use crate::notifier::Notifier;
use bollard::system::EventsOptions;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Monitor {
    config: Config,
    notifier: Notifier,
    log_alert_cache: log_watcher::LogAlertCache,
    restart_tracker: self_healer::RestartTracker,
}

impl Monitor {
    pub fn new(config: Config) -> Self {
        let notifier = Notifier::new(config.clone());
        let log_alert_cache = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let restart_tracker = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        Self {
            config,
            notifier,
            log_alert_cache,
            restart_tracker,
        }
    }

    pub async fn run(&self) -> Result<(), String> {
        println!("Connecting to Docker daemon (socket / DOCKER_HOST)...");
        let docker = crate::utils::connect_docker()?;

        docker
            .ping()
            .await
            .map_err(|e| format!("Failed to ping Docker daemon (is it running?): {}", e))?;

        println!("Successfully connected to Docker. Starting event monitor loop...");

        let docker_stats_clone = docker.clone();
        let config_clone = self.config.clone();
        let notifier_clone = self.notifier.clone();
        tokio::spawn(async move {
            if let Err(e) = resource_analyzer::run_resource_monitor(
                docker_stats_clone,
                config_clone,
                notifier_clone,
            )
            .await
            {
                eprintln!("Resource monitor error: {}", e);
            }
        });

        let list_options = Some(bollard::container::ListContainersOptions::<String> {
            all: false,
            ..Default::default()
        });
        if let Ok(active_containers) = docker.list_containers(list_options).await {
            for c in active_containers {
                if let Some(id) = c.id {
                    let name = c
                        .names
                        .as_ref()
                        .and_then(|names| names.first())
                        .map(|n| n.trim_start_matches('/'))
                        .unwrap_or("unknown");

                    if name == "unknown" {
                        continue;
                    }

                    if !self.config.is_container_monitored(name) {
                        continue;
                    }

                    let docker_clone = docker.clone();
                    let config_clone = self.config.clone();
                    let notifier_clone = self.notifier.clone();
                    let cache_clone = self.log_alert_cache.clone();
                    let id_clone = id.clone();
                    let name_clone = name.to_string();
                    tokio::spawn(async move {
                        let _ = log_watcher::monitor_container_logs(
                            docker_clone,
                            id_clone,
                            name_clone,
                            config_clone,
                            notifier_clone,
                            cache_clone,
                        )
                        .await;
                    });
                }
            }
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

        while let Some(event_res) = events_stream.next().await {
            match event_res {
                Ok(event) => {
                    if let Err(e) = event_reactor::handle_docker_event(
                        &docker,
                        &self.config,
                        &self.notifier,
                        &self.log_alert_cache,
                        &self.restart_tracker,
                        event,
                    )
                    .await
                    {
                        eprintln!("Error handling event: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error reading Docker event stream: {}", e);
                }
            }
        }

        Ok(())
    }
}
