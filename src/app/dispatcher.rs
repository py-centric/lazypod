use crate::action::Action;
use crate::app::{App, Tab};

/// Dispatch asynchronous data query to refresh all containers, images, volumes, networks, and pods.
pub fn trigger_refresh_data(app: &mut App) {
    let engines = app.get_active_engines();
    let client = app.engine_client.clone();
    let tx = app.action_tx.clone();

    tokio::spawn(async move {
        let (containers_res, images_res, volumes_res, networks_res, pods_res) = tokio::join!(
            client.get_containers(&engines),
            client.get_images(&engines),
            client.get_volumes(&engines),
            client.get_networks(&engines),
            client.get_pods(&engines)
        );

        let (running, stopped) = match containers_res {
            Ok(c) => (
                c.iter().filter(|x| x.is_running()).cloned().collect(),
                c.iter().filter(|x| !x.is_running()).cloned().collect(),
            ),
            Err(e) => {
                if let Some(ref tx) = tx {
                    let _ = tx.send(Action::Error {
                        message: format!("Containers: {e}"),
                    });
                }
                (Vec::new(), Vec::new())
            }
        };

        let images = match images_res {
            Ok(v) => v,
            Err(e) => {
                if let Some(ref tx) = tx {
                    let _ = tx.send(Action::Error {
                        message: format!("Images: {e}"),
                    });
                }
                Vec::new()
            }
        };

        let volumes = match volumes_res {
            Ok(v) => v,
            Err(e) => {
                if let Some(ref tx) = tx {
                    let _ = tx.send(Action::Error {
                        message: format!("Volumes: {e}"),
                    });
                }
                Vec::new()
            }
        };

        let networks = match networks_res {
            Ok(v) => v,
            Err(e) => {
                if let Some(ref tx) = tx {
                    let _ = tx.send(Action::Error {
                        message: format!("Networks: {e}"),
                    });
                }
                Vec::new()
            }
        };

        let pods = match pods_res {
            Ok(v) => v,
            Err(e) => {
                if let Some(ref tx) = tx {
                    let _ = tx.send(Action::Error {
                        message: format!("Pods: {e}"),
                    });
                }
                Vec::new()
            }
        };

        if let Some(tx) = tx {
            let _ = tx.send(Action::DataRefreshed {
                running,
                stopped,
                images,
                volumes,
                networks,
                pods,
            });
        }
    });
}

/// Fetch logs for the currently selected container or pod.
pub fn trigger_fetch_logs(app: &mut App) {
    let (engine, id, is_pod) = match app.active_tab {
        Tab::Running => {
            if let Some(c) = app.running.get(app.selected_index) {
                (Some(c.engine.clone()), Some(c.id.clone()), false)
            } else {
                (None, None, false)
            }
        }
        Tab::Stopped => {
            if let Some(c) = app.stopped.get(app.selected_index) {
                (Some(c.engine.clone()), Some(c.id.clone()), false)
            } else {
                (None, None, false)
            }
        }
        Tab::Pods => {
            if let Some(p) = app.pods.get(app.selected_index) {
                (Some(p.engine.clone()), Some(p.id.clone()), true)
            } else {
                (None, None, false)
            }
        }
        _ => (None, None, false),
    };

    if let (Some(engine), Some(id)) = (engine, id) {
        let client = app.engine_client.clone();
        let tx = app.action_tx.clone();
        tokio::spawn(async move {
            let logs = if is_pod {
                client.get_pod_logs(&engine, &id).await.unwrap_or_default()
            } else {
                client
                    .get_container_logs(&engine, &id)
                    .await
                    .unwrap_or_default()
            };
            if let Some(tx) = tx {
                let _ = tx.send(Action::LogsRefreshed { logs });
            }
        });
    } else {
        app.container_logs.clear();
        app.logs_state.select(None);
    }
}

/// Trigger an inspect query for the currently selected resource.
pub fn trigger_inspect(app: &mut App) {
    let resource = app.get_selected_resource();
    if let Some((_, engine, id)) = resource {
        let client = app.engine_client.clone();
        let tx = app.action_tx.clone();
        tokio::spawn(async move {
            let result = client.get_container_inspect(&engine, &id).await;
            if let Some(tx) = tx {
                match result {
                    Ok(output) => {
                        let _ = tx.send(Action::InspectResult { output });
                    }
                    Err(e) => {
                        let _ = tx.send(Action::Error {
                            message: e.to_string(),
                        });
                    }
                }
            }
        });
    }
}

/// Request a resource action (e.g. stop, start, rm), asking for confirmation if destructive.
pub fn handle_action(app: &mut App, action: &str) {
    if action == "stop" || action == "rm" {
        if let Some((tab, engine, id)) = app.get_selected_resource() {
            app.pending_action = Some((tab, engine, id, action.to_string()));
            app.show_confirmation = true;
        }
        return;
    }

    if let Some((tab, engine, id)) = app.get_selected_resource() {
        spawn_resource_action(app, tab, engine, id, action.to_string());
    }
}

/// Execute a confirmed resource action.
pub fn execute_resource_action(
    app: &App,
    resource_type: Tab,
    engine: String,
    id: String,
    action: String,
) {
    spawn_resource_action(app, resource_type, engine, id, action);
}

/// Spawn the async task for a resource action.
pub fn spawn_resource_action(app: &App, tab: Tab, engine: String, id: String, action: String) {
    let client = app.engine_client.clone();
    let tx = app.action_tx.clone();
    tokio::spawn(async move {
        let result = match tab {
            Tab::Running | Tab::Stopped => client.action_container(&engine, &id, &action).await,
            Tab::Images => client.action_image(&engine, &id, &action).await,
            Tab::Volumes => client.action_volume(&engine, &id, &action).await,
            Tab::Networks => client.action_network(&engine, &id, &action).await,
            Tab::Pods => client.action_pod(&engine, &id, &action).await,
        };
        if let Some(tx) = tx {
            match result {
                Ok(()) => {
                    let _ = tx.send(Action::ActionComplete);
                }
                Err(e) => {
                    let _ = tx.send(Action::Error {
                        message: e.to_string(),
                    });
                }
            }
        }
    });
}

/// Submit container creation form.
pub fn submit_create_container(app: &mut App) {
    if let Some(form) = app.create_container_form.take() {
        if let Some(img) = app.images.get(app.selected_index) {
            let target_engine = img.engine.clone();
            let client = app.engine_client.clone();
            let tx = app.action_tx.clone();
            let img_id = img.id.clone();
            tokio::spawn(async move {
                let result = client
                    .run_container(
                        &target_engine,
                        &img_id,
                        &form.name,
                        &form.ports,
                        &form.env,
                        &form.command,
                    )
                    .await;
                if let Some(tx) = tx {
                    match result {
                        Ok(()) => {
                            let _ = tx.send(Action::ActionComplete);
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error {
                                message: e.to_string(),
                            });
                        }
                    }
                }
            });
        }
    }
}

/// Submit pod creation form.
pub fn submit_create_pod(app: &mut App) {
    if let Some(form) = app.create_pod_form.take() {
        let engine = app.get_default_target_engine();
        let client = app.engine_client.clone();
        let tx = app.action_tx.clone();

        let mut shares = Vec::new();
        if form.share_pid {
            shares.push("pid");
        }
        if form.share_net {
            shares.push("net");
        }
        let share_str = shares.join(",");

        tokio::spawn(async move {
            let result = client
                .create_pod(&engine, &form.name, &form.network, &share_str)
                .await;
            if let Some(tx) = tx {
                match result {
                    Ok(()) => {
                        let _ = tx.send(Action::ActionComplete);
                    }
                    Err(e) => {
                        let _ = tx.send(Action::Error {
                            message: e.to_string(),
                        });
                    }
                }
            }
        });
    }
}

/// Direct image pull task.
pub fn pull_image_direct(app: &mut App, img: String) {
    app.is_pulling = true;
    let target = app.get_default_target_engine();
    let client = app.engine_client.clone();
    let tx = app.action_tx.clone();
    tokio::spawn(async move {
        let _ = client.pull_image(&target, &img).await;
        if let Some(tx) = tx {
            let _ = tx.send(Action::PullComplete);
        }
    });
}

/// Image search query task.
pub fn search_images(app: &mut App, query: String) {
    if let Some(form) = &mut app.search_image_form {
        form.is_searching = true;
    }
    let engines = app.get_active_engines();
    let client = app.engine_client.clone();
    let tx = app.action_tx.clone();
    tokio::spawn(async move {
        if let Ok(results) = client.search_images(&engines, &query).await {
            if let Some(tx) = tx {
                let _ = tx.send(Action::SearchResults { results });
            }
        }
    });
}

/// Configure unqualified search registries task.
pub fn configure_registries(app: &mut App, registries: String) {
    let client = app.engine_client.clone();
    let tx = app.action_tx.clone();
    tokio::spawn(async move {
        let _ = client.configure_registries(&registries).await;
        if let Some(tx) = tx {
            let _ = tx.send(Action::ActionComplete);
        }
    });
    app.configure_registries_form = None;
}
