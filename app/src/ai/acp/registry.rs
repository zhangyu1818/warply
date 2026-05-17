use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use warpui::{Entity, ModelContext, SingletonEntity};

pub const DEFAULT_AGENT_ID: &str = "codex-acp";
const REGISTRY_URL: &str = "https://cdn.agentclientprotocol.com/registry/v1/latest/registry.json";
const CACHE_FILE_NAME: &str = "acp-registry.json";

#[derive(Clone, Debug)]
pub enum AcpRegistryEvent {
    Updated,
}

pub struct AcpRegistryModel {
    registry: AcpRegistry,
}

impl AcpRegistryModel {
    pub fn new(ctx: &mut ModelContext<Self>) -> Self {
        let registry = load_cached_registry().unwrap_or_else(AcpRegistry::fallback);
        ctx.spawn(refresh_registry(), |model, result, ctx| match result {
            Ok(registry) => {
                if model.registry != registry {
                    if let Err(err) = save_cached_registry(&registry) {
                        log::warn!("ACP registry cache update failed: {err:#}");
                    }
                    model.registry = registry;
                    ctx.emit(AcpRegistryEvent::Updated);
                    ctx.notify();
                }
            }
            Err(err) => {
                log::warn!("ACP registry refresh failed: {err:#}");
            }
        });
        Self { registry }
    }

    pub fn registry(&self) -> &AcpRegistry {
        &self.registry
    }

    #[cfg(test)]
    pub fn new_for_test(_ctx: &mut ModelContext<Self>) -> Self {
        Self {
            registry: AcpRegistry::fallback(),
        }
    }
}

impl Entity for AcpRegistryModel {
    type Event = AcpRegistryEvent;
}

impl SingletonEntity for AcpRegistryModel {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpRegistry {
    pub version: String,
    #[serde(default)]
    pub agents: Vec<AcpRegistryAgent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpRegistryAgent {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub distribution: AcpDistribution,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpDistribution {
    #[serde(default)]
    pub binary: HashMap<String, AcpBinaryDistribution>,
    #[serde(default)]
    pub npx: Option<AcpPackageDistribution>,
    #[serde(default)]
    pub uvx: Option<AcpPackageDistribution>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpBinaryDistribution {
    pub archive: String,
    pub cmd: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpPackageDistribution {
    pub package: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpAgentLaunch {
    pub agent_id: String,
    pub display_name: String,
    pub command_line: Vec<String>,
    pub env: Vec<(String, String)>,
    pub install_command: String,
}

impl AcpRegistry {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        serde_json::from_str(value).context("failed to parse ACP registry")
    }

    pub fn fallback() -> Self {
        Self {
            version: "fallback".to_string(),
            agents: vec![AcpRegistryAgent {
                id: DEFAULT_AGENT_ID.to_string(),
                name: "Codex CLI".to_string(),
                version: "fallback".to_string(),
                description: Some("ACP adapter for OpenAI's coding assistant".to_string()),
                repository: Some("https://github.com/zed-industries/codex-acp".to_string()),
                website: None,
                authors: Vec::new(),
                license: Some("Apache-2.0".to_string()),
                icon: None,
                distribution: AcpDistribution {
                    npx: Some(AcpPackageDistribution {
                        package: "@zed-industries/codex-acp".to_string(),
                        args: Vec::new(),
                        env: HashMap::new(),
                    }),
                    ..Default::default()
                },
            }],
        }
    }

    pub fn agent(&self, id: &str) -> Option<&AcpRegistryAgent> {
        self.agents.iter().find(|agent| agent.id == id)
    }

    pub fn selectable_agents(&self) -> Vec<&AcpRegistryAgent> {
        self.agents
            .iter()
            .filter(|agent| agent.launch().is_some())
            .collect()
    }

    pub fn launch_for_agent(&self, id: &str) -> Option<AcpAgentLaunch> {
        self.agent(id)
            .and_then(AcpRegistryAgent::launch)
            .or_else(|| {
                if id != DEFAULT_AGENT_ID {
                    return None;
                }

                self.agent(DEFAULT_AGENT_ID)
                    .and_then(AcpRegistryAgent::launch)
                    .or_else(|| {
                        AcpRegistry::fallback()
                            .agent(DEFAULT_AGENT_ID)
                            .and_then(AcpRegistryAgent::launch)
                    })
            })
    }
}

impl AcpRegistryAgent {
    fn launch(&self) -> Option<AcpAgentLaunch> {
        if let Some(npx) = &self.distribution.npx {
            return Some(package_launch(
                self,
                "npx",
                vec!["-y".to_string(), npx.package.clone()],
                &npx.args,
                &npx.env,
                format!("npm i -g {}", npx.package),
            ));
        }

        if let Some(uvx) = &self.distribution.uvx {
            return Some(package_launch(
                self,
                "uvx",
                vec![uvx.package.clone()],
                &uvx.args,
                &uvx.env,
                format!("uv tool install {}", uvx.package),
            ));
        }

        let binary = self.distribution.binary.get(binary_platform())?;
        let mut command_line = vec![normalize_binary_command(&binary.cmd)];
        command_line.extend(binary.args.clone());
        Some(AcpAgentLaunch {
            agent_id: self.id.clone(),
            display_name: self.name.clone(),
            command_line,
            env: sorted_env(&binary.env),
            install_command: format!("install {} and ensure {} is on PATH", self.name, binary.cmd),
        })
    }
}

fn package_launch(
    agent: &AcpRegistryAgent,
    executable: &str,
    mut command_line: Vec<String>,
    args: &[String],
    env: &HashMap<String, String>,
    install_command: String,
) -> AcpAgentLaunch {
    command_line.insert(0, executable.to_string());
    command_line.extend(args.iter().cloned());
    AcpAgentLaunch {
        agent_id: agent.id.clone(),
        display_name: agent.name.clone(),
        command_line,
        env: sorted_env(env),
        install_command,
    }
}

fn sorted_env(env: &HashMap<String, String>) -> Vec<(String, String)> {
    let mut env = env
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect::<Vec<_>>();
    env.sort_by(|a, b| a.0.cmp(&b.0));
    env
}

fn normalize_binary_command(command: &str) -> String {
    let command = command.strip_prefix("./").unwrap_or(command);
    Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command)
        .to_string()
}

fn binary_platform() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "darwin-aarch64"
    } else {
        "darwin-x86_64"
    }
}

async fn refresh_registry() -> anyhow::Result<AcpRegistry> {
    let body = reqwest::Client::new()
        .get(REGISTRY_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    AcpRegistry::parse(&body)
}

fn registry_cache_path() -> PathBuf {
    warp_core::paths::cache_dir().join(CACHE_FILE_NAME)
}

fn load_cached_registry() -> Option<AcpRegistry> {
    let body = std::fs::read_to_string(registry_cache_path()).ok()?;
    AcpRegistry::parse(&body).ok()
}

fn save_cached_registry(registry: &AcpRegistry) -> anyhow::Result<()> {
    let path = registry_cache_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_vec_pretty(registry)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_REGISTRY_JSON: &str = r#"{
        "version": "1.0.0",
        "agents": [
            {
                "id": "test-npx-agent",
                "name": "Test NPX Agent",
                "version": "1.0.0",
                "distribution": {
                    "npx": {
                        "package": "@example/test-agent@1.0.0",
                        "args": ["--acp"]
                    }
                }
            },
            {
                "id": "test-binary-agent",
                "name": "Test Binary Agent",
                "version": "2.0.0",
                "distribution": {
                    "binary": {
                        "darwin-aarch64": {
                            "archive": "https://example.com/test-binary.zip",
                            "cmd": "./test-binary",
                            "args": ["acp"]
                        },
                        "darwin-x86_64": {
                            "archive": "https://example.com/test-binary.zip",
                            "cmd": "./test-binary",
                            "args": ["acp"]
                        }
                    }
                }
            },
            {
                "id": "test-uvx-agent",
                "name": "Test UVX Agent",
                "version": "3.0.0",
                "distribution": {
                    "uvx": {
                        "package": "example-test-agent",
                        "args": ["acp"]
                    }
                }
            }
        ]
    }"#;

    #[test]
    fn parses_registry_agents() {
        let registry = AcpRegistry::parse(TEST_REGISTRY_JSON).unwrap();

        assert_eq!(registry.agents.len(), 3);
        assert_eq!(
            registry.agent("test-uvx-agent").unwrap().name,
            "Test UVX Agent"
        );
    }

    #[test]
    fn resolves_npx_launch() {
        let registry = AcpRegistry::parse(TEST_REGISTRY_JSON).unwrap();
        let launch = registry.launch_for_agent("test-npx-agent").unwrap();

        assert_eq!(launch.display_name, "Test NPX Agent");
        assert_eq!(
            launch.command_line,
            vec![
                "npx".to_string(),
                "-y".to_string(),
                "@example/test-agent@1.0.0".to_string(),
                "--acp".to_string()
            ]
        );
    }

    #[test]
    fn resolves_binary_launch_as_path_command() {
        let registry = AcpRegistry::parse(TEST_REGISTRY_JSON).unwrap();
        let launch = registry.launch_for_agent("test-binary-agent").unwrap();

        assert_eq!(
            launch.command_line,
            vec!["test-binary".to_string(), "acp".to_string()]
        );
    }

    #[test]
    fn resolves_uvx_launch() {
        let registry = AcpRegistry::parse(TEST_REGISTRY_JSON).unwrap();
        let launch = registry.launch_for_agent("test-uvx-agent").unwrap();

        assert_eq!(
            launch.command_line,
            vec![
                "uvx".to_string(),
                "example-test-agent".to_string(),
                "acp".to_string()
            ]
        );
    }

    #[test]
    fn fallback_registry_contains_codex_for_default_agent() {
        let registry = AcpRegistry::fallback();
        let launch = registry.launch_for_agent(DEFAULT_AGENT_ID).unwrap();

        assert_eq!(launch.agent_id, DEFAULT_AGENT_ID);
        assert_eq!(launch.display_name, "Codex CLI");
    }

    #[test]
    fn missing_agent_does_not_fallback_to_codex() {
        let registry = AcpRegistry::fallback();

        assert!(registry.launch_for_agent("missing").is_none());
    }
}
