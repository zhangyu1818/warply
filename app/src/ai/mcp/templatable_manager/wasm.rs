use warpui::AppContext;
use warpui::ModelContext;

use super::TemplatableMCPServerManager;
use crate::ai::mcp::templatable::{CloudTemplatableMCPServer, TemplatableMCPServer};
use crate::ai::mcp::templatable_installation::TemplatableMCPServerInstallation;
use std::collections::HashMap;
use uuid::Uuid;

impl TemplatableMCPServerManager {
    /// Creates a new [`TemplatableMCPServerManager`] instance.
    pub fn new(
        _locally_installed_servers: HashMap<Uuid, TemplatableMCPServerInstallation>,
        _running_server_uuids: Vec<Uuid>,
        _ctx: &mut ModelContext<Self>,
    ) -> Self {
        Default::default()
    }

    /// Gets a CloudTemplatableMCPServer by its UUID.
    /// Returns the CloudTemplatableMCPServer model if found, otherwise None.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn get_cloud_templatable_mcp_server(
        &self,
        _uuid: Uuid,
    ) -> Option<&CloudTemplatableMCPServer> {
        log::warn!("Getting a CloudTemplatableMCPServer by UUID is not supported in WASM");
        None
    }

    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn get_all_templatable_mcp_servers(&self) -> Vec<&TemplatableMCPServer> {
        log::warn!("Getting all TemplatableMCPServers is not supported in WASM");
        vec![]
    }

    /// Gets a TemplatableMCPServer by its UUID.
    /// Returns the TemplatableMCPServer model if found, otherwise None.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn get_templatable_mcp_server(&self, _uuid: Uuid) -> Option<&TemplatableMCPServer> {
        log::warn!("Getting a TemplatableMCPServer by UUID is not supported in WASM");
        None
    }

    /// Spawns a new MCP server from a given [`TemplatableMCPServer`] instance.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn spawn_server(&mut self, _uuid: Uuid, _ctx: &mut ModelContext<Self>) {
        log::warn!("MCP server spawning not supported in WASM");
    }

    /// Shuts down a running MCP server.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn shutdown_server(&mut self, _uuid: Uuid, _ctx: &mut ModelContext<Self>) {
        log::warn!("MCP server shutdown not supported in WASM");
    }

    pub fn get_all_templatable_mcp_server_names(_ctx: &AppContext) -> HashMap<Uuid, String> {
        Default::default()
    }

    pub fn get_mcp_name(_uuid: &Uuid, _app: &AppContext) -> Option<String> {
        Default::default()
    }

    pub fn spawn_ephemeral_server(
        &mut self,
        _installation: TemplatableMCPServerInstallation,
        _ctx: &mut ModelContext<Self>,
    ) {
        log::warn!("Ephemeral MCP server spawning not supported in WASM");
    }
}
