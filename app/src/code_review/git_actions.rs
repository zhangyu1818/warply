use std::path::Path;

use crate::code_review::diff_state::CommitChainMode;
use crate::util::git::{self, Commit, PrInfo};

pub async fn run_commit_chain(
    repo_path: &Path,
    mode: CommitChainMode,
    message: &str,
    include_unstaged: bool,
    branch: &str,
    _autogenerate_pr_content: bool,
    path_env: Option<&str>,
) -> anyhow::Result<(Vec<Commit>, Option<String>, Option<PrInfo>)> {
    git::run_commit(repo_path, message, include_unstaged, path_env).await?;
    let pr_info = match mode {
        CommitChainMode::CommitOnly => None,
        CommitChainMode::CommitAndPush => {
            git::run_push(repo_path, branch, path_env).await?;
            None
        }
        CommitChainMode::CommitAndCreatePr => {
            git::run_push(repo_path, branch, path_env).await?;
            Some(git::create_pr(repo_path, None, None, path_env).await?)
        }
    };
    let (commits, upstream_ref) = git::compute_unpushed_state(repo_path).await;
    Ok((commits, upstream_ref, pr_info))
}

pub async fn run_push(
    repo_path: &Path,
    branch: &str,
    path_env: Option<&str>,
) -> anyhow::Result<(Vec<Commit>, Option<String>)> {
    git::run_push(repo_path, branch, path_env).await?;
    Ok(git::compute_unpushed_state(repo_path).await)
}

pub async fn create_pr(
    repo_path: &Path,
    _branch: &str,
    _autogenerate_content: bool,
    path_env: Option<&str>,
) -> anyhow::Result<PrInfo> {
    git::create_pr(repo_path, None, None, path_env).await
}

pub async fn get_pr(repo_path: &Path, path_env: Option<&str>) -> anyhow::Result<Option<PrInfo>> {
    git::get_pr_for_branch(repo_path, path_env).await
}

pub async fn get_committed_branch_files(
    repo_path: &Path,
) -> anyhow::Result<Vec<git::FileChangeEntry>> {
    git::get_committed_branch_file_entries(repo_path).await
}

pub async fn generate_commit_message(
    _repo_path: &Path,
    _branch_name: &str,
    _include_unstaged: bool,
) -> anyhow::Result<String> {
    anyhow::bail!("code review commit-message generation requires a configured local provider")
}
