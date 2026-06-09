use std::{collections::HashMap, sync::Arc, time::Duration};

use miette::{Result, miette};
use tokio::sync::{mpsc, oneshot, watch};

use crate::{
    daemon::{
        DAEMON_HOST_DISPLAY, DaemonControlCommand, DaemonLifecycleHandle, DaemonLifecycleState,
        DaemonLock, DaemonServerStartParams, SessionTokenStore, session, session_client_for_id,
        session_ipc, spawn_detached_daemon_process, start_server,
    },
    dashboard::{
        DashboardControlCommand, DashboardRuntimeActivity, DashboardRuntimeActivityStatus,
        DashboardRuntimeStatusLevel, DashboardState, dashboard_agent_name, sync_web_activity_state,
    },
    events::{EventStatus, TelegramIncomingEvent},
    runtime::bootstrap::{bootstrap_telegram_transport_state_from_acl, emit_startup_progress},
    telegram_acl::TelegramAclHandle,
    telegram_transport::{
        TelegramDeliveryClient, TelegramInputRouter, TelegramTransport,
        state::{PendingOutboundMessage, TelegramTransportState},
    },
};

struct ManagerTelegramInputRouter {
    sessions: session::SessionRegistry,
    session_tokens: SessionTokenStore,
    telegram_defaults: session::TelegramSessionDefaults,
}

#[async_trait::async_trait]
impl TelegramInputRouter for ManagerTelegramInputRouter {
    async fn route_telegram_event(&self, event: TelegramIncomingEvent) -> Result<()> {
        let chat_id = event.chat_id.clone();
        let session_id = match self.telegram_defaults.get(&chat_id) {
            Some(session_id) if self.sessions.get(&session_id).is_some() => session_id,
            _ => {
                let info = self
                    .sessions
                    .create(
                        session::SessionScope::General,
                        Some(format!("Telegram {}", event.chat_title.trim())),
                    )
                    .await?;
                self.telegram_defaults
                    .set(chat_id.clone(), info.session_id.clone())
                    .await?;
                info.session_id
            }
        };
        let client =
            session_client_for_id(&self.sessions, &self.session_tokens, session_id.as_str())
                .await?;
        match client
            .request(session_ipc::SessionIpcRequest::EnqueueTelegramEvent { event })
            .await?
        {
            session_ipc::SessionIpcResponse::Submitted { .. } => Ok(()),
            session_ipc::SessionIpcResponse::Error { message, .. } => {
                Err(miette!("session rejected telegram event: {message}"))
            }
            _ => Err(miette!("unexpected session IPC telegram route response")),
        }
    }
}

pub(crate) async fn run_daemon_serve(config: crate::config::Config) -> Result<()> {
    let mut lock = DaemonLock::acquire().await?;
    let daemon_token_registry = crate::daemon::load_or_create_daemon_token_registry().await?;
    let daemon_lifecycle = DaemonLifecycleHandle::new(DaemonLifecycleState::Initializing);

    let telegram_acl = TelegramAclHandle::load().await;
    let sessions = session::SessionRegistry::load().await?;
    let telegram_defaults = session::TelegramSessionDefaults::load().await?;
    let session_tokens: SessionTokenStore = Arc::new(parking_lot::RwLock::new(HashMap::new()));
    hydrate_session_tokens(&sessions, &session_tokens).await;
    let telegram_sessions = sessions.clone();
    let telegram_session_tokens = session_tokens.clone();
    let (dashboard_tx, _dashboard_rx) = watch::channel(manager_dashboard_state(&telegram_acl));
    let (dashboard_control_tx, mut dashboard_control_rx) =
        mpsc::unbounded_channel::<DashboardControlCommand>();
    let (daemon_control_tx, mut daemon_control_rx) =
        mpsc::unbounded_channel::<DaemonControlCommand>();
    let (server_shutdown_tx, server_shutdown_rx) = oneshot::channel();

    let daemon_server = start_server(DaemonServerStartParams {
        port: config.daemon.port,
        auth_registry: daemon_token_registry,
        lifecycle: daemon_lifecycle.clone(),
        dashboard_rx: dashboard_tx.subscribe(),
        telegram_acl: telegram_acl.clone(),
        dashboard_control_tx: dashboard_control_tx.clone(),
        daemon_control_tx: daemon_control_tx.clone(),
        sessions: sessions.clone(),
        session_tokens: session_tokens.clone(),
        shutdown_rx: server_shutdown_rx,
    })
    .await?;
    emit_startup_progress(format!(
        "[manager] listening on http://{}:{}",
        DAEMON_HOST_DISPLAY, daemon_server.port
    ));

    tokio::spawn(async {
        if let Err(err) = crate::model_catalog::refresh_models_dev_cache().await {
            tracing::warn!("models.dev cache refresh failed: {err}");
        }
    });

    let telegram_transport = if config.telegram.enabled && config.telegram.has_real_credentials() {
        let telegram = TelegramTransportState::new();
        let telegram_handle = telegram.handle();
        bootstrap_telegram_transport_state_from_acl(&telegram_handle, &telegram_acl);
        Some(tokio::spawn(
            TelegramTransport::new(
                config.telegram.clone(),
                telegram_handle,
                telegram_acl.clone(),
                Arc::new(ManagerTelegramInputRouter {
                    sessions: telegram_sessions,
                    session_tokens: telegram_session_tokens,
                    telegram_defaults,
                }),
                dashboard_tx.subscribe(),
                dashboard_control_tx.clone(),
            )
            .run(),
        ))
    } else {
        None
    };
    let telegram_outbox_delivery =
        if config.telegram.enabled && config.telegram.has_real_credentials() {
            Some(tokio::spawn(run_session_telegram_outbox_delivery(
                TelegramDeliveryClient::new(config.telegram.clone(), telegram_acl.clone()),
                sessions.clone(),
                session_tokens.clone(),
            )))
        } else {
            None
        };
    let session_health_checks = tokio::spawn(run_session_health_checks(
        sessions.clone(),
        session_tokens.clone(),
    ));

    daemon_lifecycle.mark_ready();

    #[cfg(unix)]
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|err| miette!("failed to install SIGTERM handler: {err}"))?;
    let mut ctrl_c_disabled = false;
    let mut restart_requested = false;
    let mut shutdown_completion_tx = None;

    loop {
        tokio::select! {
            Some(command) = daemon_control_rx.recv() => {
                apply_daemon_control_command(
                    command,
                    &mut shutdown_completion_tx,
                    &mut restart_requested,
                );
                break;
            }
            Some(command) = dashboard_control_rx.recv() => {
                match command {
                    DashboardControlCommand::RestartDaemon => {
                        restart_requested = true;
                        break;
                    }
                    DashboardControlCommand::RunSleep => {
                        tracing::warn!("manager received sleep run command, but sleep runs inside sessions");
                    }
                    DashboardControlCommand::ClearConversation => {
                        tracing::warn!("manager received clear conversation command, but conversation state is session-scoped");
                    }
                }
            }
            signal = tokio::signal::ctrl_c(), if !ctrl_c_disabled => {
                match signal {
                    Ok(()) => {
                        tracing::info!("manager received SIGINT, shutting down");
                        break;
                    }
                    Err(err) => {
                        tracing::warn!("ctrl_c listener failed: {err}");
                        ctrl_c_disabled = true;
                    }
                }
            }
            _ = {
                #[cfg(unix)] { sigterm.recv() }
                #[cfg(not(unix))] { std::future::pending::<Option<()>>() }
            } => {
                tracing::info!("manager received SIGTERM, shutting down");
                break;
            }
        }
    }

    daemon_lifecycle.mark_stopping();
    if let Some(handle) = telegram_transport {
        handle.abort();
    }
    if let Some(handle) = telegram_outbox_delivery {
        handle.abort();
    }
    session_health_checks.abort();
    lock.release();
    if let Some(completion_tx) = shutdown_completion_tx.take() {
        let _ = completion_tx.send(());
    }
    drop(dashboard_tx);
    let _ = server_shutdown_tx.send(());
    let _ = tokio::time::timeout(Duration::from_secs(15), daemon_server.shutdown()).await;
    if restart_requested {
        spawn_detached_daemon_process().await?;
    }
    Ok(())
}

async fn run_session_health_checks(
    sessions: session::SessionRegistry,
    session_tokens: SessionTokenStore,
) {
    loop {
        for info in sessions.list() {
            if !info.status.is_process_backed() {
                continue;
            }
            let Some(ipc_name) = info.ipc_name.clone() else {
                let _ = sessions.mark_dead(&info.session_id).await;
                continue;
            };
            let Some(ipc_token) = session_tokens.read().get(&info.session_id).cloned() else {
                let _ = sessions.mark_dead(&info.session_id).await;
                continue;
            };
            let client =
                session_ipc::SessionIpcClient::new(info.session_id.clone(), ipc_name, ipc_token)
                    .with_timeout(Duration::from_secs(2));
            match client.request(session_ipc::SessionIpcRequest::Status).await {
                Ok(session_ipc::SessionIpcResponse::Status { runtime_status })
                    if runtime_status.ready =>
                {
                    let _ = sessions.mark_ready(&info.session_id).await;
                }
                Ok(session_ipc::SessionIpcResponse::Status { .. }) => {}
                Ok(session_ipc::SessionIpcResponse::Error { message, .. }) => {
                    tracing::warn!("session {} health check error: {message}", info.session_id);
                }
                Ok(_) => {}
                Err(err) => {
                    tracing::warn!("session {} health check failed: {err:?}", info.session_id);
                    session_tokens.write().remove(&info.session_id);
                    let _ = sessions.mark_dead(&info.session_id).await;
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn run_session_telegram_outbox_delivery(
    delivery: TelegramDeliveryClient,
    sessions: session::SessionRegistry,
    session_tokens: SessionTokenStore,
) {
    loop {
        for info in sessions.list() {
            if !info.status.is_process_backed()
                || !session_tokens.read().contains_key(&info.session_id)
            {
                continue;
            }
            let client =
                match session_client_for_id(&sessions, &session_tokens, info.session_id.as_str())
                    .await
                {
                    Ok(client) => client,
                    Err(err) => {
                        tracing::debug!(
                            "skip telegram outbox drain for session {}: {err:?}",
                            info.session_id
                        );
                        continue;
                    }
                };
            let messages = match client
                .request(session_ipc::SessionIpcRequest::DrainTelegramOutbox)
                .await
            {
                Ok(session_ipc::SessionIpcResponse::TelegramOutbox { messages }) => messages,
                Ok(session_ipc::SessionIpcResponse::Error { message, .. }) => {
                    tracing::warn!(
                        "session {} rejected telegram outbox drain: {message}",
                        info.session_id
                    );
                    continue;
                }
                Ok(_) => {
                    tracing::warn!(
                        "session {} returned unexpected telegram outbox response",
                        info.session_id
                    );
                    continue;
                }
                Err(err) => {
                    tracing::debug!(
                        "telegram outbox drain failed for session {}: {err:?}",
                        info.session_id
                    );
                    continue;
                }
            };

            for message in messages {
                if let Err(err) =
                    deliver_session_telegram_message(&client, &delivery, message).await
                {
                    tracing::warn!(
                        "telegram delivery failed for session {}: {err:?}",
                        info.session_id
                    );
                    break;
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn deliver_session_telegram_message(
    client: &session_ipc::SessionIpcClient,
    delivery: &TelegramDeliveryClient,
    message: PendingOutboundMessage,
) -> Result<()> {
    match delivery.send_pending_outbound(&message).await {
        Ok(()) => {
            if let Some(event_id) = message.related_event_id.as_deref() {
                record_telegram_delivery(
                    client,
                    event_id,
                    message
                        .settle_status_on_delivery
                        .unwrap_or(EventStatus::Resolved),
                    message.settle_note_on_delivery.clone(),
                )
                .await?;
            }
            Ok(())
        }
        Err(err) => {
            let reason = format!("{err:?}");
            if let Some(event_id) = message.related_event_id.as_deref() {
                let _ = record_telegram_delivery(
                    client,
                    event_id,
                    EventStatus::AwaitingDelivery,
                    Some(reason.clone()),
                )
                .await;
            }
            requeue_telegram_outbound(client, message).await?;
            Err(miette!("telegram outbound delivery failed: {reason}"))
        }
    }
}

async fn record_telegram_delivery(
    client: &session_ipc::SessionIpcClient,
    event_id: &str,
    status: EventStatus,
    note: Option<String>,
) -> Result<()> {
    match client
        .request(session_ipc::SessionIpcRequest::RecordTelegramDelivery {
            event_id: event_id.to_string(),
            status,
            note,
        })
        .await?
    {
        session_ipc::SessionIpcResponse::DeliveryRecorded => Ok(()),
        session_ipc::SessionIpcResponse::Error { message, .. } => {
            Err(miette!("record telegram delivery failed: {message}"))
        }
        _ => Err(miette!("unexpected telegram delivery record response")),
    }
}

async fn requeue_telegram_outbound(
    client: &session_ipc::SessionIpcClient,
    message: PendingOutboundMessage,
) -> Result<()> {
    match client
        .request(session_ipc::SessionIpcRequest::RequeueTelegramOutbound { message })
        .await?
    {
        session_ipc::SessionIpcResponse::TelegramOutboundRequeued => Ok(()),
        session_ipc::SessionIpcResponse::Error { message, .. } => {
            Err(miette!("requeue telegram outbound failed: {message}"))
        }
        _ => Err(miette!("unexpected telegram outbound requeue response")),
    }
}

async fn hydrate_session_tokens(
    sessions: &session::SessionRegistry,
    session_tokens: &SessionTokenStore,
) {
    for info in sessions.list() {
        if !info.status.is_process_backed() {
            continue;
        }
        match session::load_session_ipc_token(&info.session_id).await {
            Ok(Some(token)) => {
                if info
                    .ipc_token_hash
                    .as_deref()
                    .is_some_and(|hash| hash == session::hash_ipc_token(&token))
                {
                    session_tokens.write().insert(info.session_id, token);
                } else {
                    tracing::warn!(
                        "discarding IPC token for session {} because its hash does not match registry",
                        info.session_id
                    );
                }
            }
            Ok(None) => {}
            Err(err) => {
                tracing::warn!(
                    "failed to load IPC token for session {}: {err:?}",
                    info.session_id
                );
            }
        }
    }
}

fn manager_dashboard_state(telegram_acl: &TelegramAclHandle) -> DashboardState {
    let mut state = DashboardState {
        agent_name: dashboard_agent_name(),
        status_output:
            "Manager daemon is running.\nSelect or create a session to view runtime state."
                .to_string(),
        inspect_telegram_output: manager_telegram_status_output(telegram_acl),
        pending_access_requests: telegram_acl.pending_requests(),
        runtime_status: Some("Manager ready".to_string()),
        runtime_status_level: Some(DashboardRuntimeStatusLevel::Info),
        runtime_activity: DashboardRuntimeActivity::new(
            DashboardRuntimeActivityStatus::Idle,
            "Manager",
            Some("Routing session traffic".to_string()),
        ),
        footer_context:
            "Manager daemon: session runtime state is available through selected sessions."
                .to_string(),
        ..DashboardState::default()
    };
    sync_web_activity_state(&mut state);
    state
}

fn manager_telegram_status_output(telegram_acl: &TelegramAclHandle) -> String {
    let pending = telegram_acl.pending_requests();
    if pending.is_empty() {
        return "Telegram ACL: no pending access requests".to_string();
    }

    let mut lines = vec!["Telegram ACL pending access requests:".to_string()];
    lines.extend(pending.into_iter().map(|request| {
        format!(
            "  {} | {} | {} | {}",
            request.chat_id, request.title, request.sender, request.last_message_preview
        )
    }));
    lines.join("\n")
}

fn apply_daemon_control_command(
    command: DaemonControlCommand,
    shutdown_completion_tx: &mut Option<oneshot::Sender<()>>,
    restart_requested: &mut bool,
) {
    match command {
        DaemonControlCommand::ShutdownRequested { completion_tx } => {
            *shutdown_completion_tx = Some(completion_tx);
        }
        DaemonControlCommand::RestartRequested => {
            *restart_requested = true;
        }
    }
}
