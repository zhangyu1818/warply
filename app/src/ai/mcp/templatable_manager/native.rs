use crate::ai::mcp::file_based_manager::FileBasedMCPManagerEvent;
use crate::ai::mcp::FileBasedMCPManager;
use futures::FutureExt as _;
use itertools::Itertools;
use std::{collections::HashMap, future::Future};

use crate::ai::mcp::http_client::build_client_with_headers;
use crate::ai::mcp::parsing::resolve_json;
use crate::ai::mcp::templatable_manager::FigmaMcpStatus;
use crate::ai::mcp::{MCPServer, TemplatableMCPServer};
use crate::cloud_object::model::persistence::{CloudModel, CloudModelEvent};
use crate::{
    ai::mcp::{
        logs, templatable::CloudTemplatableMCPServer, StaticEnvVar,
        TemplatableMCPServerInstallation, TransportType,
    },
    cloud_object::{GenericStringObjectFormat, JsonObjectType},
    drive::CloudObjectTypeAndId,
    persistence::ModelEvent,
    settings::AISettings,
    view_components::DismissibleToast,
    workspace::ToastStack,
    GlobalResourceHandlesProvider,
};
use async_compat::CompatExt as _;
use cfg_if::cfg_if;
use rmcp::{transport::ConfigureCommandExt as _, ServiceExt as _};
use simple_logger::manager::LogManager;
use simple_logger::SimpleLogger;
use tokio::io::AsyncBufReadExt as _;
use uuid::Uuid;
use warp_core::safe_error;
use warp_core::{execution_mode::AppExecutionMode, settings::Setting as _};
use warpui::AppContext;
use warpui::{windowing::WindowManager, ModelContext, SingletonEntity};

use super::{
    MCPServerState, SpawnedServerInfo, TemplatableMCPServerInfo, TemplatableMCPServerManager,
    TemplatableMCPServerManagerEvent,
};

/// Controls the behavior of `spawn_server_impl`.
enum SpawnMode {
    Initial {
        /// Whether to persist running state to SQLite.
        persist_running_state_to_sqlite: bool,
    },
    Reconnect,
}

impl SpawnMode {
    fn should_persist_running_state_to_sqlite(&self) -> bool {
        matches!(
            self,
            SpawnMode::Initial {
                persist_running_state_to_sqlite: true
            }
        )
    }

    fn is_reconnect(&self) -> bool {
        matches!(self, SpawnMode::Reconnect)
    }
}

/// Convert an rmcp error to a user-friendly error message.
fn error_to_user_message(error: &rmcp::RmcpError) -> String {
    match error {
        rmcp::RmcpError::ClientInitialize(err) => {
            format!("Failed to initialize client: {}", err)
        }
        rmcp::RmcpError::ServerInitialize(err) => {
            format!("Failed to initialize server: {}", err)
        }
        rmcp::RmcpError::TransportCreation { error, .. } => {
            format!("Failed to establish connection: {}", error)
        }
        rmcp::RmcpError::Runtime(err) => {
            format!("Runtime error: {}", err)
        }
        rmcp::RmcpError::Service(err) => match err {
            rmcp::ServiceError::McpError(_) => {
                "Server returned an error. Please check server logs for details.".to_string()
            }
            rmcp::ServiceError::TransportSend(_) => {
                "Failed to send data to server. Connection may have been lost.".to_string()
            }
            rmcp::ServiceError::TransportClosed => {
                "Connection closed unexpectedly. The server may have crashed.".to_string()
            }
            rmcp::ServiceError::UnexpectedResponse => {
                "Server sent an unexpected response. The server may be incompatible.".to_string()
            }
            rmcp::ServiceError::Cancelled { reason } => format!(
                "Operation was cancelled with reason: {}",
                reason.clone().unwrap_or("Unknown reason".to_string())
            ),
            rmcp::ServiceError::Timeout { timeout } => {
                format!(
                    "Connection timed out after {} seconds. The server may be unresponsive.",
                    timeout.as_secs()
                )
            }
            _ => format!("Service error: {}", err),
        },
    }
}

/// An MCP server integration that Warp ships with bundled skills for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpIntegration {
    Figma,
}

impl TemplatableMCPServerManager {
    /// Returns `true` if the given MCP integration is currently running.
    pub fn is_mcp_server_running(&self, integration: McpIntegration) -> bool {
        match integration {
            McpIntegration::Figma => self.get_figma_mcp_status() == FigmaMcpStatus::Running,
        }
    }

    /// Creates a new [`TemplatableMCPServerManager`] instance.
    pub fn new(
        locally_installed_servers: HashMap<Uuid, TemplatableMCPServerInstallation>,
        running_server_uuids: Vec<Uuid>,
        ctx: &mut ModelContext<Self>,
    ) -> Self {
        // Subscribe to FileBasedMCPManager events.
        let file_based_mcp_manager = FileBasedMCPManager::handle(ctx);
        ctx.subscribe_to_model(&file_based_mcp_manager, |me, event, ctx| match event {
            FileBasedMCPManagerEvent::SpawnServers { installations } => {
                me.spawn_file_based_servers(installations, ctx);
            }
            FileBasedMCPManagerEvent::DespawnServers { installation_uuids } => {
                me.despawn_file_based_servers(installation_uuids.clone(), ctx);
            }
        });

        let cloud_model = CloudModel::handle(ctx);
        ctx.subscribe_to_model(&cloud_model, |me, event, ctx| match event {
            CloudModelEvent::ObjectUpdated {
                type_and_id:
                    CloudObjectTypeAndId::GenericStringObject {
                        object_type:
                            GenericStringObjectFormat::Json(JsonObjectType::TemplatableMCPServer),
                        id: _,
                    },
            }
            | CloudModelEvent::ObjectTrashed {
                type_and_id:
                    CloudObjectTypeAndId::GenericStringObject {
                        object_type:
                            GenericStringObjectFormat::Json(JsonObjectType::TemplatableMCPServer),
                        id: _,
                    },
            }
            | CloudModelEvent::ObjectUntrashed {
                type_and_id:
                    CloudObjectTypeAndId::GenericStringObject {
                        object_type:
                            GenericStringObjectFormat::Json(JsonObjectType::TemplatableMCPServer),
                        id: _,
                    },
            }
            | CloudModelEvent::ObjectDeleted {
                type_and_id:
                    CloudObjectTypeAndId::GenericStringObject {
                        object_type:
                            GenericStringObjectFormat::Json(JsonObjectType::TemplatableMCPServer),
                        id: _,
                    },
                folder_id: _,
            }
            | CloudModelEvent::ObjectMoved {
                type_and_id:
                    CloudObjectTypeAndId::GenericStringObject {
                        object_type:
                            GenericStringObjectFormat::Json(JsonObjectType::TemplatableMCPServer),
                        id: _,
                    },
                from_folder: _,
                to_folder: _,
            } => {
                me.fetch_cloud_servers(ctx);
            }
            CloudModelEvent::ObjectCreated {
                type_and_id:
                    CloudObjectTypeAndId::GenericStringObject {
                        object_type:
                            GenericStringObjectFormat::Json(JsonObjectType::TemplatableMCPServer),
                        id: new_sync_id,
                    },
            } => {
                log::debug!("A new MCP server template was found with sync id {new_sync_id}");
                me.fetch_cloud_servers(ctx);
            }
            _ => {}
        });

        let mut me = Self {
            cloud_templatable_mcp_servers: Default::default(),
            server_states: Default::default(),
            active_servers: Default::default(),
            spawned_servers: Default::default(),
            locally_installed_servers,
            server_error_messages: Default::default(),
            spawner: Some(ctx.spawner()),
            pending_reconnections: Default::default(),
        };

        me.fetch_cloud_servers(ctx);

        if AppExecutionMode::as_ref(ctx).can_autostart_mcp_servers() {
            for installation_uuid in running_server_uuids {
                me.spawn_server(installation_uuid, ctx)
            }
        }

        me
    }

    fn delete_orphaned_installations(&mut self, ctx: &mut ModelContext<Self>) {
        let orphaned_installations: Vec<Uuid> = self.locally_installed_servers
            .iter()
            .filter(|(_, installation)| {
                // Gallery-sourced installations don't have a corresponding cloud template
                // and should never be treated as orphans.
                installation.gallery_uuid().is_none()
                    && !self.cloud_templatable_mcp_servers.contains_key(&installation.template_uuid())
            })
            .map(|(installation_uuid, installation)| {
                log::info!("Deleting orphaned MCP server installation {installation_uuid} named {} with no corresponding cloud template {}", installation.templatable_mcp_server().name, installation.template_uuid());
                *installation_uuid
            })
            .collect();

        self.delete_templatable_mcp_server_installations(orphaned_installations, ctx);
    }

    fn fetch_cloud_servers(&mut self, ctx: &mut ModelContext<Self>) {
        self.cloud_templatable_mcp_servers = Self::get_cloud_servers(ctx);
        self.delete_orphaned_installations(ctx);
        ctx.emit(TemplatableMCPServerManagerEvent::TemplatableMCPServersUpdated);
    }

    fn get_cloud_servers(ctx: &mut ModelContext<Self>) -> HashMap<Uuid, CloudTemplatableMCPServer> {
        let cloud_templatable_mcp_servers: Vec<CloudTemplatableMCPServer> =
            CloudTemplatableMCPServer::get_all(ctx);
        cloud_templatable_mcp_servers
            .into_iter()
            .map(|server| (server.model().string_model.uuid, server))
            .collect()
    }

    pub fn change_server_state(
        &mut self,
        installation_uuid: Uuid,
        new_state: MCPServerState,
        ctx: &mut ModelContext<Self>,
    ) {
        // If a server is rapidly stopped and started there is a race condition
        // Checking the server's actual status helps us avoid this
        if matches!(new_state, MCPServerState::NotRunning)
            && (self.is_server_active(installation_uuid)
                || self.is_server_active_or_pending(installation_uuid))
        {
            return;
        }
        self.server_states.insert(installation_uuid, new_state);
        ctx.emit(TemplatableMCPServerManagerEvent::StateChanged);
    }

    /// Gets a CloudTemplatableMCPServer by its UUID.
    /// Returns the CloudTemplatableMCPServer model if found, otherwise None.
    pub fn get_cloud_templatable_mcp_server(
        &self,
        uuid: Uuid,
    ) -> Option<&CloudTemplatableMCPServer> {
        self.cloud_templatable_mcp_servers.get(&uuid)
    }

    /// Gets a TemplatableMCPServer by its UUID.
    /// Returns the TemplatableMCPServer model if found, otherwise None.
    pub fn get_templatable_mcp_server(&self, uuid: Uuid) -> Option<&TemplatableMCPServer> {
        self.get_cloud_templatable_mcp_server(uuid)
            .map(|server| &server.model().string_model)
    }

    pub fn get_all_templatable_mcp_servers(&self) -> Vec<&TemplatableMCPServer> {
        self.cloud_templatable_mcp_servers
            .values()
            .map(|server| &server.model().string_model)
            .collect()
    }

    pub fn get_all_templatable_mcp_server_names(ctx: &AppContext) -> HashMap<Uuid, String> {
        TemplatableMCPServerManager::as_ref(ctx)
            .get_all_templatable_mcp_servers()
            .iter()
            .map(|&server| (server.uuid, server.name.clone()))
            .collect()
    }

    /// Get the name for an MCP server based on uuid.
    pub fn get_mcp_name(uuid: &Uuid, app: &AppContext) -> Option<String> {
        TemplatableMCPServerManager::as_ref(app)
            .get_installed_server_name(uuid)
            .or_else(|| TemplatableMCPServerManager::as_ref(app).get_template_server_name(uuid))
    }

    fn get_installed_server_name(&self, installation_uuid: &Uuid) -> Option<String> {
        self.get_installed_server(installation_uuid)
            .map(|server| server.templatable_mcp_server().name.clone())
    }

    fn get_template_server_name(&self, template_uuid: &Uuid) -> Option<String> {
        self.get_templatable_mcp_server(*template_uuid)
            .map(|template| template.name.clone())
    }

    fn persist_is_mcp_running(
        installation_uuid: Uuid,
        running: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        let global_resource_handles = GlobalResourceHandlesProvider::as_ref(ctx).get();

        let Some(sender) = &global_resource_handles.model_event_sender else {
            return;
        };
        let event = ModelEvent::UpdateMCPInstallationRunning {
            installation_uuid,
            running,
        };
        if let Err(err) = sender.send(event) {
            log::error!(
                "Failed to save TemplatableMCPServerInstallation running status to database: {err}"
            );
        }
    }

    /// Spawns an ephemeral MCP server from a given [`TemplatableMCPServerInstallation`].
    ///
    /// Unlike `spawn_server`, this method takes the installation directly and does not
    /// persist it to the database. The server exists only for the duration of the current
    /// session (e.g., an agent CLI run).
    pub fn spawn_ephemeral_server(
        &mut self,
        installation: TemplatableMCPServerInstallation,
        ctx: &mut ModelContext<Self>,
    ) {
        let installation_uuid = installation.uuid();
        log::debug!("Spawning ephemeral server with installation_uuid {installation_uuid}");

        self.spawn_server_impl(
            installation,
            SpawnMode::Initial {
                persist_running_state_to_sqlite: false,
            },
            ctx,
        );
    }

    /// Spawns a new MCP server from a given installation UUID.
    ///
    /// This looks up the installation from `locally_installed_servers` and persists
    /// running state changes to the database.
    ///
    /// Depending on the configuration, this will either spawn a child process for a server
    /// configured to use the stdio transport or start a Streamable HTTP transport client.
    ///
    /// The server will be started in the background and the result will be handled by the
    /// [`TemplatableMCPServerManager`].
    pub fn spawn_server(&mut self, installation_uuid: Uuid, ctx: &mut ModelContext<Self>) {
        log::debug!("Trying to spawn a server with installation_uuid {installation_uuid}");

        // Look up installation to resolve template variables and use its UUID as the MCP server id.
        let Some(installation) = self
            .locally_installed_servers
            .get(&installation_uuid)
            .cloned()
        else {
            log::error!(
                "No templatable MCP installation found for installation_uuid {installation_uuid}; cannot resolve template variables"
            );

            self.change_server_state(installation_uuid, MCPServerState::FailedToStart, ctx);
            return;
        };

        self.spawn_server_impl(
            installation,
            SpawnMode::Initial {
                persist_running_state_to_sqlite: true,
            },
            ctx,
        );
    }

    /// Internal implementation of server spawning.
    fn spawn_server_impl(
        &mut self,
        installation: TemplatableMCPServerInstallation,
        mode: SpawnMode,
        ctx: &mut ModelContext<Self>,
    ) {
        let installation_uuid = installation.uuid();

        let resolved_json = resolve_json(&installation);
        let template_uuid = installation.template_uuid();

        let mut server = match MCPServer::from_user_json(&resolved_json) {
            Ok(mut servers) => match servers.pop() {
                Some(mut s) => {
                    // We want to use installation uuid, not template uuid.
                    s.uuid = installation_uuid;
                    s
                }
                None => {
                    log::error!(
                        "Templatable MCP server template contains no servers: {template_uuid}",
                    );
                    self.change_server_state(installation_uuid, MCPServerState::FailedToStart, ctx);
                    if mode.is_reconnect() {
                        self.notify_reconnect_waiters(
                            installation_uuid,
                            Err("Template contains no servers".to_string()),
                        );
                    }
                    return;
                }
            },
            Err(err) => {
                log::error!(
                    "Failed to parse resolved MCP server JSON for '{template_uuid}': {err:#}",
                );
                self.change_server_state(installation_uuid, MCPServerState::FailedToStart, ctx);
                if mode.is_reconnect() {
                    self.notify_reconnect_waiters(
                        installation_uuid,
                        Err(format!("Failed to parse MCP server: {err:#}")),
                    );
                }
                return;
            }
        };

        // If we're executing a CLI MCP server, ensure that the environment variables includes
        // PATH.
        if let TransportType::CLIServer(cli_server) = &mut server.transport_type {
            let Some(execution_path) = AISettings::as_ref(ctx).mcp_execution_path.value().clone()
            else {
                // This can only happen if the user is trying to launch an MCP server
                // without ever having had a successfully bootstrapped session, which
                // should basically never happen.
                log::warn!("Unknown PATH when trying to launch MCP command.");

                self.change_server_state(installation_uuid, MCPServerState::FailedToStart, ctx);

                if let Some(window_id) = WindowManager::as_ref(ctx).active_window() {
                    ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                        toast_stack.add_ephemeral_toast(
                            DismissibleToast::error(
                                "PATH required to launch MCP server. Please open a new terminal session to autopopulate PATH."
                                    .to_string(),
                            ),
                            window_id,
                            ctx,
                        );
                    });
                }

                if mode.is_reconnect() {
                    self.notify_reconnect_waiters(
                        installation_uuid,
                        Err("PATH not available".to_string()),
                    );
                }
                return;
            };

            // Prepend our PATH to the static env vars, in case the user has
            // specified a custom PATH in the MCP server settings.
            cli_server.static_env_vars.insert(
                0,
                StaticEnvVar {
                    name: "PATH".to_string(),
                    value: execution_path,
                },
            );

            // For file-based MCP installations without an explicit `working_directory`,
            // default the spawn cwd to the directory the config was discovered in
            // (repo root for project-scoped configs, ~/.warp/ or ~ for globals). This
            // matches user expectations for repo-relative commands in `.mcp.json`.
            // Cloud-templated installations (lookup returns None) are unaffected and
            // continue to inherit Warp's process cwd.
            if cli_server.cwd_parameter.is_none() {
                if let Some(spawn_root) =
                    FileBasedMCPManager::as_ref(ctx).spawn_root_for_installation(installation_uuid)
                {
                    cli_server.cwd_parameter = Some(spawn_root.to_string_lossy().into_owned());
                }
            }
        }

        let executor = ctx.background_executor().clone();
        let logger = match LogManager::handle(ctx).update(ctx, |mgr, _| {
            mgr.register_namespace("mcp", true);
            mgr.register(
                "mcp",
                logs::relative_log_file_path_from_uuid(&template_uuid),
                executor,
            )
        }) {
            Ok(logger) => logger,
            Err(e) => {
                safe_error!(
                    safe: ("Failed to register MCP log file: {}", e.safe_message()),
                    full: ("Failed to register MCP log file for {template_uuid}: {e}")
                );
                return;
            }
        };
        let logger_clone = logger.clone();

        let server_name = server.name.clone();
        let description = installation.templatable_mcp_server().description.clone();

        // Extract values from mode before moving it into the closure.
        let should_persist = mode.should_persist_running_state_to_sqlite();
        let is_reconnect = mode.is_reconnect();

        self.change_server_state(installation_uuid, MCPServerState::Starting, ctx);
        let task = ctx.spawn(
            spawn_server(
                server_name,
                description,
                installation_uuid,
                server.transport_type.clone(),
                logger.clone(),
            )
            .compat(),
            move |me, server_info: Result<_, rmcp::RmcpError>, ctx| {
                me.spawned_servers.remove(&installation_uuid);

                match server_info {
                    Ok(info) => {
                        let peer = info.service.clone();
                        me.active_servers.insert(installation_uuid, info);

                        // Clear any previous error message on successful connection.
                        me.server_error_messages.remove(&installation_uuid);

                        if should_persist {
                            ctx.dispatch_global_action("workspace:save_app", ());
                            Self::persist_is_mcp_running(installation_uuid, true, ctx);
                        }
                        me.change_server_state(installation_uuid, MCPServerState::Running, ctx);

                        if is_reconnect {
                            me.notify_reconnect_waiters(installation_uuid, Ok(peer));
                        }
                    }
                    Err(e) => {
                        logger_clone
                            .log(format!("[error] MCP: Failed to connect to server: {e:#}"));
                        // Close the logger to make sure we flush any remaining data.
                        logger_clone.close();
                        log::warn!("Failed to spawn MCP server: {e:#}");

                        // Store user-friendly error message.
                        let error_message = error_to_user_message(&e);
                        me.server_error_messages
                            .insert(installation_uuid, error_message.clone());

                        me.change_server_state(
                            installation_uuid,
                            MCPServerState::FailedToStart,
                            ctx,
                        );

                        if is_reconnect {
                            me.notify_reconnect_waiters(installation_uuid, Err(error_message));
                        }
                    }
                };
            },
        );

        self.spawned_servers.insert(
            installation_uuid,
            SpawnedServerInfo {
                abort_handle: task.abort_handle(),
            },
        );

        log::debug!(
            "We have successfully spawned a server with installation_uuid {installation_uuid}!"
        );
    }

    /// Shuts down a running MCP server.
    pub fn shutdown_server(&mut self, installation_uuid: Uuid, ctx: &mut ModelContext<'_, Self>) {
        log::debug!("Trying to shut down a MCP server with installation uuid {installation_uuid}");

        Self::persist_is_mcp_running(installation_uuid, false, ctx);

        // There's 2 possibilities:
        // 1. The server is still connecting, in which case we cancel the spawn
        // 2. The server is already running, in which case we shutdown the server
        // We do both to avoid race conditions and have to do it in this order to avoid the server connecting between the shutdown_server and cancel_spawn calls
        if let Some(spawned_info) = self.spawned_servers.remove(&installation_uuid) {
            spawned_info.abort_handle.abort();
        }
        if let Some(server_info) = self.active_servers.remove(&installation_uuid) {
            self.change_server_state(installation_uuid, MCPServerState::ShuttingDown, ctx);
            // Cancel the server, and emit NotRunning state once it has stopped.
            ctx.spawn(server_info.service.cancel(), move |me, _, ctx| {
                me.change_server_state(installation_uuid, MCPServerState::NotRunning, ctx);
                ctx.dispatch_global_action("workspace:save_app", ());
            });
        } else {
            self.change_server_state(installation_uuid, MCPServerState::NotRunning, ctx);
        }

        log::debug!("Successfully shut down server with installation uuid {installation_uuid}");
    }

    fn delete_templatable_mcp_server_installations(
        &mut self,
        installation_uuids: Vec<Uuid>,
        ctx: &mut ModelContext<Self>,
    ) {
        if installation_uuids.is_empty() {
            return;
        }

        for installation_uuid in &installation_uuids {
            self.shutdown_server(*installation_uuid, ctx);

            // Delete log files using template_uuid
            if let Some(installation) = self.locally_installed_servers.get(installation_uuid) {
                let template_uuid = installation.template_uuid();
                let log_file_path = logs::log_file_path_from_uuid(&template_uuid);
                if log_file_path.exists() {
                    let _ = std::fs::remove_file(log_file_path);
                }
            }

            self.locally_installed_servers.remove(installation_uuid);
        }

        // Delete the entries from the local database
        let global_resource_handles = GlobalResourceHandlesProvider::as_ref(ctx).get();

        if let Some(sender) = &global_resource_handles.model_event_sender {
            let event = ModelEvent::DeleteMCPServerInstallations {
                installation_uuids: installation_uuids.clone(),
            };
            if let Err(err) = sender.send(event) {
                log::error!("Failed to delete installations from local database: {err}");
            }
        }

        ctx.emit(TemplatableMCPServerManagerEvent::ServerInstallationDeleted);
        ctx.notify();
    }

    /// Returns the peer for the given installation UUID if it is connected and the transport is not closed.
    pub fn get_peer_if_connected(
        &self,
        installation_uuid: Uuid,
    ) -> Option<rmcp::Peer<rmcp::RoleClient>> {
        self.active_servers
            .get(&installation_uuid)
            .and_then(|server| {
                if server.service.is_transport_closed() {
                    None
                } else {
                    Some(server.service.clone())
                }
            })
    }

    /// Triggers reconnection of a server by its installation UUID.
    ///
    /// If a reconnection is already in progress for this server, the caller is added to the
    /// waiting list and will be notified when the existing reconnection completes.
    /// Otherwise, a new reconnection is started.
    ///
    /// The result is sent via the provided oneshot channel when the connection completes (or fails).
    pub fn reconnect_server(
        &mut self,
        installation_uuid: Uuid,
        result_tx: tokio::sync::oneshot::Sender<Result<rmcp::Peer<rmcp::RoleClient>, String>>,
        ctx: &mut ModelContext<Self>,
    ) {
        log::debug!("Reconnecting MCP server with installation uuid {installation_uuid}");

        // If a reconnection is already in progress, add this caller to the waiting list.
        if let Some(waiters) = self.pending_reconnections.get_mut(&installation_uuid) {
            log::debug!(
                "Reconnection already in progress for {installation_uuid}, adding to waiters"
            );
            waiters.push(result_tx);
            return;
        }

        // Start tracking this reconnection with this caller as the first waiter.
        self.pending_reconnections
            .insert(installation_uuid, vec![result_tx]);

        // Remove the old server from active_servers if it exists.
        self.active_servers.remove(&installation_uuid);

        // Cancel any in-flight spawn.
        if let Some(spawned_info) = self.spawned_servers.remove(&installation_uuid) {
            spawned_info.abort_handle.abort();
        }

        // Look up the installation to get server details.
        let Some(installation) = self
            .locally_installed_servers
            .get(&installation_uuid)
            .cloned()
        else {
            self.notify_reconnect_waiters(
                installation_uuid,
                Err("Installation not found".to_string()),
            );
            return;
        };

        self.spawn_server_impl(installation, SpawnMode::Reconnect, ctx);
    }

    /// Notifies all pending reconnection waiters for the given installation UUID.
    ///
    /// This removes the waiters from `pending_reconnections` and sends the result to each.
    fn notify_reconnect_waiters(
        &mut self,
        installation_uuid: Uuid,
        result: Result<rmcp::Peer<rmcp::RoleClient>, String>,
    ) {
        if let Some(waiters) = self.pending_reconnections.remove(&installation_uuid) {
            for tx in waiters {
                // Clone the result for each waiter. For Ok, we clone the peer.
                // For Err, we clone the error message.
                let _ = tx.send(result.clone());
            }
        }
    }

    /// Returns a reconnecting peer for a server that has the given tool.
    ///
    /// The returned peer will automatically reconnect if the underlying transport is closed.
    pub fn server_with_tool_name(
        &self,
        tool_name: String,
    ) -> Option<crate::ai::mcp::reconnecting_peer::ReconnectingPeer> {
        let spawner = self.spawner.as_ref()?;
        self.active_servers
            .iter()
            .find(|(_, server)| server.tools.iter().any(|t| t.name == tool_name))
            .map(|(installation_uuid, _)| {
                crate::ai::mcp::reconnecting_peer::ReconnectingPeer::new(
                    *installation_uuid,
                    spawner.clone(),
                )
            })
    }

    /// Returns a reconnecting peer for a server with the given installation ID and tool.
    ///
    /// The returned peer will automatically reconnect if the underlying transport is closed.
    pub fn server_with_installation_id_and_tool_name(
        &self,
        installation_id: Uuid,
        tool_name: String,
    ) -> Option<crate::ai::mcp::reconnecting_peer::ReconnectingPeer> {
        let spawner = self.spawner.as_ref()?;
        let server = self.active_servers.get(&installation_id)?;
        if server.tools.iter().any(|t| t.name == tool_name) {
            Some(crate::ai::mcp::reconnecting_peer::ReconnectingPeer::new(
                installation_id,
                spawner.clone(),
            ))
        } else {
            None
        }
    }

    fn spawn_file_based_servers(
        &mut self,
        installations: &[TemplatableMCPServerInstallation],
        ctx: &mut ModelContext<Self>,
    ) {
        // First, check if the servers are already spawned.
        let new_installations = installations
            .iter()
            .filter(|installation| {
                let uuid = installation.uuid();
                !self.active_servers.contains_key(&uuid)
                    && !self.spawned_servers.contains_key(&uuid)
            })
            .cloned()
            .collect_vec();

        // If not, spawn them.
        for installation in new_installations {
            self.spawn_ephemeral_server(installation, ctx);
        }
    }

    fn despawn_file_based_servers(
        &mut self,
        installation_uuids: Vec<Uuid>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.delete_templatable_mcp_server_installations(installation_uuids, ctx);
    }
}

type ReqwestHttpTransport = rmcp::transport::StreamableHttpClientTransport<reqwest::Client>;
type ReqwestSseTransport = rmcp::transport::SseClientTransport<reqwest::Client>;

/// Spawns a new MCP server from a given [`TransportType`].
async fn spawn_server(
    server_name: String,
    _description: Option<String>,
    _uuid: Uuid,
    transport_type: TransportType,
    logger: SimpleLogger,
) -> Result<TemplatableMCPServerInfo, rmcp::RmcpError> {
    logger.log("[note] Attention! There may be sensitive information (such as API keys) in these logs. Make sure to redact any secrets before sharing with others.".to_string());

    let service = match transport_type {
        TransportType::CLIServer(cli_server) => {
            logger.log("[info] MCP: Using stdio transport".to_string());

            cfg_if! {
                if #[cfg(windows)] {
                    // We wrap the command in cmd.exe /c to allow Windows to be responsible for resolving the
                    // PATH variable rather than depending on the `Command` implementation, which only looks for
                    // `.exe` files in directories found in PATH.
                    // https://github.com/rust-lang/rust/issues/37519
                    let command = "cmd.exe".to_owned();
                    let args = std::iter::once("/c".to_owned())
                        .chain(std::iter::once(cli_server.command))
                        .chain(cli_server.args)
                        .collect::<Vec<String>>();
                } else {
                    let command = cli_server.command;
                    let args = cli_server.args;
                }
            }

            // Capture the command and configured cwd for diagnostics before they're
            // moved into the Command builder closure.
            let command_for_log = command.clone();
            let cwd_for_log = cli_server.cwd_parameter.clone();

            // Try to spawn the child process.
            let (transport, stderr) = rmcp::transport::TokioChildProcess::builder(
                tokio::process::Command::new(command).configure(|cmd| {
                    cmd.args(args);
                    if let Some(cwd) = cli_server.cwd_parameter {
                        cmd.current_dir(cwd);
                    }
                    for StaticEnvVar { name, value } in cli_server.static_env_vars.iter() {
                        if value.is_empty() {
                            // Skip empty/unset environment variables so that, in the CLI, they can be inherited.
                            logger.log(format!(
                                "[warn] MCP: Skipping empty environment variable: {name}"
                            ));
                            continue;
                        }
                        cmd.env(name, value);
                    }

                    // On Windows, ensure that no console window is shown.
                    #[cfg(windows)]
                    cmd.creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0);
                }),
            )
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    let cwd_display = cwd_for_log
                        .as_deref()
                        .unwrap_or("<inherited from Warp's process cwd>");
                    logger.log(format!(
                        "[error] MCP: Failed to spawn '{server_name}': command '{command_for_log}' \
                         not found (cwd: {cwd_display}). If your MCP server depends on a specific \
                         working directory, set the `working_directory` field in your config to \
                         override the default."
                    ));
                }
                rmcp::RmcpError::transport_creation::<rmcp::transport::TokioChildProcess>(err)
            })?;

            let pid = transport
                .id()
                .map(|pid| pid.to_string())
                .unwrap_or("??".to_string());

            // We always expect to have an stderr, but this is marginally safer than unwrapping.
            if let Some(stderr) = stderr {
                let logger = logger.clone();
                // Spawn a background task to forward from the child process's stderr to our logger.
                tokio::spawn(async move {
                    let mut buf = String::new();
                    let mut reader = tokio::io::BufReader::new(stderr);
                    loop {
                        match reader.read_line(&mut buf).await {
                            // EOF.
                            Ok(0) => return,
                            // Read some data.
                            Ok(_) => logger.log(format!("[info] MCP [pid: {pid}] stderr: {buf}")),
                            // Failed to read from the child process's stderr.
                            Err(e) => {
                                log::error!("Failed to read stderr: {e}");
                                return;
                            }
                        }
                    }
                });
            }

            // Wrap the transport in a logging wrapper.
            let transport = TransportLoggingWrapper {
                transport,
                logger: logger.clone(),
            };

            // Create the MCP client and connect to the server.
            Ok::<_, rmcp::RmcpError>(make_client_info().into_dyn().serve(transport).await?)
        }
        TransportType::ServerSentEvents(sse_server) => {
            let headers: std::collections::HashMap<String, String> = sse_server
                .headers
                .iter()
                .map(|h| (h.name.clone(), h.value.clone()))
                .collect();
            match determine_transport(&sse_server.url, &headers).await {
                Ok(Transport::Http) => {
                    logger.log("[info] MCP: Using Streaming HTTP transport".to_string());
                    let transport = if headers.is_empty() {
                        rmcp::transport::StreamableHttpClientTransport::from_uri(
                            sse_server.url.clone(),
                        )
                    } else {
                        let client = build_client_with_headers(&headers)?;
                        rmcp::transport::StreamableHttpClientTransport::with_client(
                            client,
                            rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig::with_uri(
                                sse_server.url.clone(),
                            ),
                        )
                    };
                    let transport = TransportLoggingWrapper {
                        transport,
                        logger: logger.clone(),
                    };
                    Ok(make_client_info().into_dyn().serve(transport).await?)
                }
                Ok(Transport::Sse) => {
                    logger.log("[info] MCP: Using (legacy) SSE transport (due to preflight failing with a 404)".to_string());
                    let transport = if headers.is_empty() {
                        rmcp::transport::SseClientTransport::start(sse_server.url.clone())
                            .await
                            .map_err(|e| {
                                rmcp::RmcpError::transport_creation::<ReqwestSseTransport>(e)
                            })?
                    } else {
                        let client = build_client_with_headers(&headers)?;
                        rmcp::transport::SseClientTransport::start_with_client(
                            client,
                            rmcp::transport::sse_client::SseClientConfig {
                                sse_endpoint: sse_server.url.clone().into(),
                                ..Default::default()
                            },
                        )
                        .await
                        .map_err(rmcp::RmcpError::transport_creation::<ReqwestSseTransport>)?
                    };
                    let transport = TransportLoggingWrapper {
                        transport,
                        logger: logger.clone(),
                    };
                    Ok(make_client_info().into_dyn().serve(transport).await?)
                }
                Err(err) => {
                    logger.log(format!(
                        "[error] MCP: preflight connection to MCP server failed: {err:#}"
                    ));
                    Err(err)?
                }
            }
        }
    }?;

    let server_info = service.peer_info();
    logger.log(format!("[info] MCP: Connected to server: {server_info:#?}"));

    let resources = if server_info.is_some_and(|info| info.capabilities.resources.is_some()) {
        match service.list_all_resources().await {
            Ok(result) => result,
            Err(err) => {
                log::warn!("Failed to list resources for MCP server '{server_name}': {err}");
                vec![]
            }
        }
    } else {
        vec![]
    };
    let tools = match service.list_all_tools().await {
        Ok(result) => result,
        Err(rmcp::ServiceError::McpError(rmcp::model::ErrorData { code, .. }))
            if code == rmcp::model::ErrorCode::METHOD_NOT_FOUND =>
        {
            vec![]
        }
        Err(err) => {
            return Err(err.into());
        }
    };

    Ok(TemplatableMCPServerInfo {
        service,
        resources,
        tools,
    })
}

/// The transport to use for MCP.
enum Transport {
    Http,
    Sse,
}

/// Determines which transport to use.
///
/// This sends a "preflight" InitializeRequest to the server to determine whether the
/// server supports the HTTP transport (or needs to use the SSE transport), and if
/// authentication is required.
async fn determine_transport(
    url: &str,
    headers: &std::collections::HashMap<String, String>,
) -> Result<Transport, rmcp::RmcpError> {
    use reqwest::StatusCode;

    fn unexpected_error(status: reqwest::StatusCode) -> rmcp::RmcpError {
        rmcp::RmcpError::transport_creation::<ReqwestHttpTransport>(format!(
            "Unexpected status code: {status}"
        ))
    }
    match send_initialize_request(url, headers).await? {
        StatusCode::OK => Ok(Transport::Http),
        StatusCode::NOT_FOUND | StatusCode::METHOD_NOT_ALLOWED => Ok(Transport::Sse),
        StatusCode::UNAUTHORIZED => {
            Err(rmcp::RmcpError::transport_creation::<ReqwestHttpTransport>(
                "MCP server requires authentication, which is not available in local-only mode."
                    .to_string(),
            ))
        }
        status => Err(unexpected_error(status)),
    }
}

/// Sends an InitializeRequest to the server, and returns the HTTP status code from the response.
async fn send_initialize_request(
    url: &str,
    headers: &std::collections::HashMap<String, String>,
) -> Result<reqwest::StatusCode, rmcp::RmcpError> {
    use rmcp::transport::common::http_header::{EVENT_STREAM_MIME_TYPE, JSON_MIME_TYPE};

    let request = rmcp::model::InitializeRequest::new(make_client_info());
    let request = rmcp::model::ClientJsonRpcMessage::request(
        rmcp::model::ClientRequest::InitializeRequest(request),
        rmcp::model::RequestId::Number(0),
    );

    let request = build_client_with_headers(headers)?
        .post(url)
        .header(
            http::header::ACCEPT,
            [EVENT_STREAM_MIME_TYPE, JSON_MIME_TYPE].join(", "),
        )
        .json(&request);

    let response = request
        .send()
        .await
        .map_err(rmcp::RmcpError::transport_creation::<ReqwestHttpTransport>)?;

    Ok(response.status())
}

/// Creates a [`ClientInfo`] for the MCP client.
///
/// This tells the MCP server who we are and what capabilities we have.
fn make_client_info() -> rmcp::model::ClientInfo {
    rmcp::model::ClientInfo {
        protocol_version: Default::default(),
        capabilities: Default::default(),
        client_info: rmcp::model::Implementation {
            name: warp_core::channel::ChannelState::app_id().to_string(),
            version: warp_core::channel::ChannelState::app_version()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            title: None,
            icons: None,
            website_url: None,
        },
    }
}

/// A wrapper around a [`rmcp::transport::Transport`] that logs all requests and responses.
struct TransportLoggingWrapper<T> {
    transport: T,
    logger: SimpleLogger,
}

impl<T: rmcp::transport::Transport<R>, R: rmcp::service::ServiceRole> rmcp::transport::Transport<R>
    for TransportLoggingWrapper<T>
{
    type Error = T::Error;

    fn send(
        &mut self,
        item: rmcp::service::TxJsonRpcMessage<R>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
        if let Ok(json) = serde_json::to_string(&item) {
            self.logger
                .log(format!("[info] MCP: Sending request: {json}"));
        }

        let logger = self.logger.clone();
        self.transport.send(item).map(move |result| {
            if let Err(e) = &result {
                logger.log(format!("[warn] MCP: Failed to send request: {e:#}"));
            }
            result
        })
    }

    fn receive(
        &mut self,
    ) -> impl Future<Output = Option<rmcp::service::RxJsonRpcMessage<R>>> + Send {
        let logger = self.logger.clone();
        async move {
            let result = self.transport.receive().await;
            if let Some(item) = &result {
                if let Ok(json) = serde_json::to_string(item) {
                    logger.log(format!("[info] MCP: Received response: {json}"));
                }
            }
            result
        }
    }

    fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.transport.close()
    }
}
