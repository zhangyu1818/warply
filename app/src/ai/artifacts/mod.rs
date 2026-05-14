pub mod buttons;
pub use buttons::{ArtifactButtonsRow, ArtifactButtonsRowEvent};

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "artifact_type", content = "data")]
pub enum Artifact {
    #[serde(rename = "PLAN")]
    Plan {
        document_uid: String,
        title: Option<String>,
    },
    #[serde(rename = "PULL_REQUEST")]
    PullRequest {
        url: String,
        branch: String,
        #[serde(skip_serializing)] // We derive this field from the url on deserialize
        repo: Option<String>,
        #[serde(skip_serializing)] // We derive this field from the url on deserialize
        number: Option<u32>,
    },
    #[serde(rename = "SCREENSHOT")]
    Screenshot {
        artifact_uid: String,
        mime_type: String,
        description: Option<String>,
    },
    #[serde(rename = "FILE")]
    File {
        artifact_uid: String,
        filepath: String,
        filename: String,
        mime_type: String,
        description: Option<String>,
        size_bytes: Option<i32>,
    },
}

#[derive(serde::Deserialize)]
#[serde(tag = "artifact_type", content = "data")]
enum ArtifactHelper {
    #[serde(rename = "PLAN")]
    Plan {
        document_uid: String,
        title: Option<String>,
    },
    #[serde(rename = "PULL_REQUEST")]
    PullRequest { url: String, branch: String },
    #[serde(rename = "SCREENSHOT")]
    Screenshot {
        artifact_uid: String,
        mime_type: String,
        description: Option<String>,
    },
    #[serde(rename = "FILE")]
    File {
        artifact_uid: String,
        filepath: String,
        filename: String,
        mime_type: String,
        description: Option<String>,
        size_bytes: Option<i32>,
    },
}

impl<'de> serde::Deserialize<'de> for Artifact {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper = ArtifactHelper::deserialize(deserializer)?;
        Ok(match helper {
            ArtifactHelper::Plan {
                document_uid,
                title,
            } => Artifact::Plan {
                document_uid,
                title,
            },
            ArtifactHelper::PullRequest { url, branch } => {
                let (repo, number) = parse_github_pr_url(&url).unzip();
                Artifact::PullRequest {
                    url,
                    branch,
                    repo,
                    number,
                }
            }
            ArtifactHelper::Screenshot {
                artifact_uid,
                mime_type,
                description,
            } => Artifact::Screenshot {
                artifact_uid,
                mime_type,
                description,
            },
            ArtifactHelper::File {
                artifact_uid,
                filepath,
                filename,
                mime_type,
                description,
                size_bytes,
            } => Artifact::File {
                artifact_uid,
                filepath,
                filename,
                mime_type,
                description,
                size_bytes,
            },
        })
    }
}

/// Parse GitHub PR URL to extract repo and number.
/// Expected format: https://github.com/{owner}/{repo}/pull/{number}
pub fn parse_github_pr_url(url: &str) -> Option<(String, u32)> {
    if !url.contains("github.com") {
        return None;
    }
    let segments: Vec<&str> = url.split('/').collect();
    segments.windows(3).find_map(|w| {
        if w[1] != "pull" {
            return None;
        }
        Some((w[0].to_string(), w[2].parse().ok()?))
    })
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
