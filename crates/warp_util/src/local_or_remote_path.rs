use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{remote_path::RemotePath, standardized_path::StandardizedPath};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum LocalOrRemotePath {
    Local(PathBuf),
    Remote(RemotePath),
}

impl LocalOrRemotePath {
    pub fn is_local(&self) -> bool {
        matches!(self, LocalOrRemotePath::Local(_))
    }

    pub fn is_remote(&self) -> bool {
        matches!(self, LocalOrRemotePath::Remote(_))
    }

    pub fn path_component(&self) -> StandardizedPath {
        match self {
            LocalOrRemotePath::Local(path) => StandardizedPath::from_local_absolute_unchecked(path),
            LocalOrRemotePath::Remote(remote) => remote.path.clone(),
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            LocalOrRemotePath::Local(path) => path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default(),
            LocalOrRemotePath::Remote(remote) => remote.path.file_name().unwrap_or_default(),
        }
    }

    pub fn display_path(&self) -> String {
        match self {
            LocalOrRemotePath::Local(path) => path.to_string_lossy().into_owned(),
            LocalOrRemotePath::Remote(remote) => remote.path.to_string(),
        }
    }

    pub fn to_local_path(&self) -> Option<&Path> {
        match self {
            LocalOrRemotePath::Local(path) => Some(path),
            LocalOrRemotePath::Remote(_) => None,
        }
    }

    pub fn join(&self, segment: &str) -> Self {
        match self {
            LocalOrRemotePath::Local(path) => Self::Local(path.join(segment)),
            LocalOrRemotePath::Remote(remote) => Self::Remote(RemotePath::new(
                remote.host_id.clone(),
                remote.path.join(segment),
            )),
        }
    }

    pub fn strip_repo_prefix(&self, file: &Self) -> Option<String> {
        match (self, file) {
            (Self::Local(repo), Self::Local(file)) => file
                .strip_prefix(repo)
                .ok()
                .map(|path| path.to_string_lossy().into_owned()),
            (Self::Remote(repo), Self::Remote(file)) if repo.host_id == file.host_id => {
                file.path.strip_prefix(&repo.path).map(str::to_owned)
            }
            _ => None,
        }
    }
}

impl From<PathBuf> for LocalOrRemotePath {
    fn from(path: PathBuf) -> Self {
        Self::Local(path)
    }
}

impl From<RemotePath> for LocalOrRemotePath {
    fn from(path: RemotePath) -> Self {
        Self::Remote(path)
    }
}

impl TryFrom<LocalOrRemotePath> for PathBuf {
    type Error = RemotePath;

    fn try_from(path: LocalOrRemotePath) -> Result<Self, Self::Error> {
        match path {
            LocalOrRemotePath::Local(path) => Ok(path),
            LocalOrRemotePath::Remote(path) => Err(path),
        }
    }
}

impl TryFrom<&LocalOrRemotePath> for PathBuf {
    type Error = ();

    fn try_from(path: &LocalOrRemotePath) -> Result<Self, Self::Error> {
        match path {
            LocalOrRemotePath::Local(path) => Ok(path.clone()),
            LocalOrRemotePath::Remote(_) => Err(()),
        }
    }
}
