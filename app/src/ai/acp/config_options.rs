use std::path::PathBuf;

use agent_client_protocol::schema::{
    InitializeRequest, NewSessionRequest, ProtocolVersion, SessionConfigKind, SessionConfigOption,
    SessionConfigOptionCategory, SessionConfigSelectOptions,
};
use agent_client_protocol::{Agent, Client, ConnectionTo};
use agent_client_protocol_tokio::AcpAgent;

use crate::settings::AcpAgentBackend;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpConfigOption {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<SessionConfigOptionCategory>,
    pub current_value: String,
    pub values: Vec<AcpConfigOptionValue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpConfigOptionValue {
    pub id: String,
    pub name: String,
}

pub fn flatten_config_options(options: &[SessionConfigOption]) -> Vec<AcpConfigOption> {
    options
        .iter()
        .filter_map(|option| {
            let SessionConfigKind::Select(select) = &option.kind else {
                return None;
            };
            let values = match &select.options {
                SessionConfigSelectOptions::Ungrouped(options) => options
                    .iter()
                    .map(|option| AcpConfigOptionValue {
                        id: option.value.0.to_string(),
                        name: option.name.clone(),
                    })
                    .collect(),
                SessionConfigSelectOptions::Grouped(groups) => groups
                    .iter()
                    .flat_map(|group| group.options.iter())
                    .map(|option| AcpConfigOptionValue {
                        id: option.value.0.to_string(),
                        name: option.name.clone(),
                    })
                    .collect(),
                _ => Vec::new(),
            };
            if values.is_empty() {
                return None;
            }

            Some(AcpConfigOption {
                id: option.id.0.to_string(),
                name: option.name.clone(),
                description: option.description.clone(),
                category: option.category.clone(),
                current_value: select.current_value.0.to_string(),
                values,
            })
        })
        .collect()
}

pub async fn probe_config_options(
    backend: AcpAgentBackend,
    cwd: PathBuf,
) -> anyhow::Result<Vec<AcpConfigOption>> {
    let agent = AcpAgent::from_args([backend.adapter_command()])?;

    let config_options = Client
        .builder()
        .connect_with(agent, |connection: ConnectionTo<Agent>| async move {
            connection
                .send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;

            let session = connection
                .send_request(NewSessionRequest::new(cwd))
                .block_task()
                .await?;
            let config_options = session
                .config_options
                .as_deref()
                .map(flatten_config_options)
                .unwrap_or_default();

            Ok(config_options)
        })
        .await?;

    Ok(config_options)
}
