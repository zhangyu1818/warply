//! Unified diff state module.
//!
//! The local and remote implementations share the code-review view contract.
//! The local implementation remains the source of truth for local git work;
//! the remote variant is the host-scoped transport-backed model.

use std::{path::{Path, PathBuf}, sync::Arc};

use crate::util::git::{Commit, FileChangeEntry, PrInfo};
use anyhow::Result;
use warp_core::SessionId;
use warp_util::remote_path::RemotePath;
use warpui::{AppContext, ModelContext, ModelHandle};

mod local;
pub use local::*;

mod remote;
pub use remote::RemoteDiffStateModel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitChainMode {
    CommitOnly,
    CommitAndPush,
    CommitAndCreatePr,
}

#[derive(Clone, Debug)]
pub enum GitOpResult {
    CommitChainCompleted(std::result::Result<Option<PrInfo>, String>),
    PushCompleted(std::result::Result<(), String>),
    PrCreated(std::result::Result<PrInfo, String>),
}

pub enum DiffStateModel {
    Local(ModelHandle<LocalDiffStateModel>),
    Remote(ModelHandle<RemoteDiffStateModel>),
}

impl warpui::Entity for DiffStateModel {
    type Event = DiffStateModelEvent;
}

impl DiffStateModel {
    pub fn new_local(repo_path: PathBuf, ctx: &mut ModelContext<Self>) -> Self {
        Self::new(Some(repo_path.display().to_string()), ctx)
    }

    pub fn new(repo_path: Option<String>, ctx: &mut ModelContext<Self>) -> Self {
        let local = ctx.add_model(|ctx| LocalDiffStateModel::new(repo_path, ctx));
        ctx.subscribe_to_model(&local, Self::forward_event);
        Self::Local(local)
    }

    pub fn new_remote(
        remote_path: RemotePath,
        session_id: SessionId,
        ctx: &mut ModelContext<Self>,
    ) -> Self {
        let remote = ctx.add_model(|ctx| {
            RemoteDiffStateModel::new(remote_path, DiffMode::default(), session_id, ctx)
        });
        ctx.subscribe_to_model(&remote, Self::forward_event);
        Self::Remote(remote)
    }

    fn forward_event(&mut self, event: &DiffStateModelEvent, ctx: &mut ModelContext<Self>) {
        ctx.emit(event.clone());
    }

    pub fn get(&self, ctx: &AppContext) -> DiffState {
        match self {
            Self::Local(model) => model.as_ref(ctx).get(),
            Self::Remote(model) => model.as_ref(ctx).get(),
        }
    }

    pub fn diff_mode(&self, ctx: &AppContext) -> DiffMode {
        match self {
            Self::Local(model) => model.as_ref(ctx).diff_mode(),
            Self::Remote(model) => model.as_ref(ctx).diff_mode(),
        }
    }

    pub fn get_uncommitted_stats(&self, ctx: &AppContext) -> Option<DiffStats> {
        match self {
            Self::Local(model) => model.as_ref(ctx).get_uncommitted_stats(),
            Self::Remote(model) => model.as_ref(ctx).get_uncommitted_stats(),
        }
    }

    pub fn uncommitted_file_entries<'a>(&self, ctx: &'a AppContext) -> &'a [FileChangeEntry] {
        match self {
            Self::Local(model) => model.as_ref(ctx).uncommitted_file_entries(),
            Self::Remote(model) => model.as_ref(ctx).uncommitted_file_entries(),
        }
    }

    pub fn get_main_branch_stats(&self, ctx: &AppContext) -> Option<DiffStats> {
        match self {
            Self::Local(model) => model.as_ref(ctx).get_main_branch_stats(),
            Self::Remote(model) => model.as_ref(ctx).get_main_branch_stats(),
        }
    }

    pub fn get_diff_stats(&self, ctx: &AppContext) -> GitDiffInfo {
        match self {
            Self::Local(model) => model.as_ref(ctx).get_diff_stats(),
            Self::Remote(model) => model.as_ref(ctx).get_diff_stats(),
        }
    }

    pub fn get_stats_for_current_mode(&self, ctx: &AppContext) -> Option<DiffStats> {
        match self {
            Self::Local(model) => model.as_ref(ctx).get_stats_for_current_mode(),
            Self::Remote(model) => model.as_ref(ctx).get_stats_for_current_mode(),
        }
    }

    pub fn get_stats_for_mode(&self, mode: DiffMode, ctx: &AppContext) -> Option<DiffStats> {
        match self {
            Self::Local(model) => model.as_ref(ctx).get_stats_for_mode(mode),
            Self::Remote(model) => model.as_ref(ctx).get_stats_for_mode(mode),
        }
    }

    pub fn diff_mode_for_base_branch(
        &self,
        base_branch: Option<&str>,
        ctx: &AppContext,
    ) -> DiffMode {
        match self {
            Self::Local(model) => model.as_ref(ctx).diff_mode_for_base_branch(base_branch),
            Self::Remote(model) => model.as_ref(ctx).diff_mode_for_base_branch(base_branch),
        }
    }

    pub fn get_current_branch_name(&self, ctx: &AppContext) -> Option<String> {
        match self {
            Self::Local(model) => model.as_ref(ctx).get_current_branch_name(),
            Self::Remote(model) => model.as_ref(ctx).get_current_branch_name(),
        }
    }

    pub fn get_main_branch_name(&self, ctx: &AppContext) -> Option<String> {
        match self {
            Self::Local(model) => model.as_ref(ctx).get_main_branch_name(),
            Self::Remote(model) => model.as_ref(ctx).get_main_branch_name(),
        }
    }

    pub fn is_on_main_branch(&self, ctx: &AppContext) -> bool {
        match self {
            Self::Local(model) => model.as_ref(ctx).is_on_main_branch(),
            Self::Remote(model) => model.as_ref(ctx).is_on_main_branch(),
        }
    }

    pub fn unpushed_commits<'a>(&self, ctx: &'a AppContext) -> &'a [Commit] {
        match self {
            Self::Local(model) => model.as_ref(ctx).unpushed_commits(),
            Self::Remote(model) => model.as_ref(ctx).unpushed_commits(),
        }
    }

    pub fn upstream_ref<'a>(&self, ctx: &'a AppContext) -> Option<&'a str> {
        match self {
            Self::Local(model) => model.as_ref(ctx).upstream_ref(),
            Self::Remote(model) => model.as_ref(ctx).upstream_ref(),
        }
    }

    pub fn upstream_differs_from_main(&self, ctx: &AppContext) -> bool {
        match self {
            Self::Local(model) => model.as_ref(ctx).upstream_differs_from_main(),
            Self::Remote(model) => model.as_ref(ctx).upstream_differs_from_main(),
        }
    }

    pub fn pr_info<'a>(&self, ctx: &'a AppContext) -> Option<&'a PrInfo> {
        match self {
            Self::Local(model) => model.as_ref(ctx).pr_info(),
            Self::Remote(model) => model.as_ref(ctx).pr_info(),
        }
    }

    pub fn is_pr_info_refreshing(&self, ctx: &AppContext) -> bool {
        match self {
            Self::Local(model) => model.as_ref(ctx).is_pr_info_refreshing(),
            Self::Remote(model) => model.as_ref(ctx).is_pr_info_refreshing(),
        }
    }

    pub fn is_git_operation_blocked(&self, ctx: &AppContext) -> bool {
        match self {
            Self::Local(model) => model.as_ref(ctx).is_git_operation_blocked(ctx),
            Self::Remote(model) => model.as_ref(ctx).is_git_operation_blocked(ctx),
        }
    }

    pub fn has_head(&self, ctx: &AppContext) -> bool {
        match self {
            Self::Local(model) => model.as_ref(ctx).has_head(),
            Self::Remote(model) => model.as_ref(ctx).has_head(),
        }
    }

    pub fn active_repository_path(&self, ctx: &AppContext) -> Option<PathBuf> {
        match self {
            Self::Local(model) => model.as_ref(ctx).active_repository_path(ctx),
            Self::Remote(model) => model.as_ref(ctx).active_repository_path(ctx),
        }
    }

    pub fn is_inside_repository(&self, ctx: &AppContext) -> bool {
        match self {
            Self::Local(model) => model.as_ref(ctx).is_inside_repository(),
            Self::Remote(model) => model.as_ref(ctx).is_inside_repository(),
        }
    }

    pub fn set_diff_mode(
        &self,
        mode: DiffMode,
        should_fetch_base: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| {
                model.set_diff_mode(mode, should_fetch_base, ctx)
            }),
            Self::Remote(model) => model.update(ctx, |model, ctx| {
                model.set_diff_mode(mode, ctx)
            }),
        }
    }

    pub fn set_diff_mode_and_fetch_base(&self, mode: DiffMode, ctx: &mut ModelContext<Self>) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| {
                model.set_diff_mode_and_fetch_base(mode, ctx)
            }),
            Self::Remote(model) => model.update(ctx, |model, ctx| {
                model.set_diff_mode(mode, ctx)
            }),
        }
    }

    pub fn load_diffs_for_current_repo(
        &self,
        should_fetch_base: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| {
                model.load_diffs_for_current_repo(should_fetch_base, ctx)
            }),
            Self::Remote(model) => model.update(ctx, |model, ctx| {
                model.load_diffs_for_current_repo(should_fetch_base, ctx)
            }),
        }
    }

    pub fn set_code_review_metadata_refresh_enabled(
        &self,
        enabled: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| {
                model.set_code_review_metadata_refresh_enabled(enabled, ctx)
            }),
            Self::Remote(model) => model.update(ctx, |model, ctx| {
                model.set_code_review_metadata_refresh_enabled(enabled, ctx)
            }),
        }
    }

    pub fn refresh_metadata_and_pr_info(&self, ctx: &mut ModelContext<Self>) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| {
                model.refresh_diff_metadata_for_current_repo(
                    InvalidationBehavior::PromptRefresh,
                    ctx,
                )
            }),
            Self::Remote(model) => {
                model.update(ctx, |model, ctx| model.refresh_metadata_and_pr_info(ctx))
            }
        }
    }

    pub fn refresh_diff_metadata_for_current_repo(
        &mut self,
        invalidation_behavior: InvalidationBehavior,
        ctx: &mut ModelContext<Self>,
    ) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| {
                model.refresh_diff_metadata_for_current_repo(invalidation_behavior, ctx)
            }),
            Self::Remote(model) => model.update(ctx, |model, ctx| {
                model.refresh_diff_metadata_for_current_repo(invalidation_behavior, ctx)
            }),
        }
    }

    pub fn refresh_pr_info(&mut self, ctx: &mut ModelContext<Self>) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| model.refresh_pr_info(ctx)),
            Self::Remote(model) => model.update(ctx, |model, ctx| model.refresh_pr_info(ctx)),
        }
    }

    pub fn git_commit_chain(
        &self,
        mode: CommitChainMode,
        message: String,
        include_unstaged: bool,
        branch: String,
        autogenerate_pr_content: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| {
                model.git_commit_chain(
                    mode,
                    message,
                    include_unstaged,
                    branch,
                    autogenerate_pr_content,
                    ctx,
                )
            }),
            Self::Remote(model) => model.update(ctx, |model, ctx| {
                model.git_commit_chain(
                    mode,
                    message,
                    include_unstaged,
                    branch,
                    autogenerate_pr_content,
                    ctx,
                )
            }),
        }
    }

    pub fn git_push(&self, branch: String, ctx: &mut ModelContext<Self>) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| model.git_push(branch, ctx)),
            Self::Remote(model) => model.update(ctx, |model, ctx| model.git_push(branch, ctx)),
        }
    }

    pub fn create_pr(
        &self,
        branch: String,
        autogenerate_content: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| {
                model.create_pr(branch, autogenerate_content, ctx)
            }),
            Self::Remote(model) => model.update(ctx, |model, ctx| {
                model.create_pr(branch, autogenerate_content, ctx)
            }),
        }
    }

    pub fn fetch_committed_branch_files(&self, ctx: &mut ModelContext<Self>) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| {
                model.fetch_committed_branch_files(ctx)
            }),
            Self::Remote(model) => model.update(ctx, |model, ctx| {
                model.fetch_committed_branch_files(ctx)
            }),
        }
    }

    pub fn generate_commit_message(
        &self,
        include_unstaged: bool,
        branch_name: String,
        ctx: &mut ModelContext<Self>,
    ) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| {
                model.generate_commit_message(include_unstaged, branch_name, ctx)
            }),
            Self::Remote(model) => model.update(ctx, |model, ctx| {
                model.generate_commit_message(include_unstaged, branch_name, ctx)
            }),
        }
    }

    pub fn discard_files(
        &self,
        file_infos: Vec<FileStatusInfo>,
        should_stash: bool,
        branch_name: Option<String>,
        ctx: &mut ModelContext<Self>,
    ) {
        match self {
            Self::Local(model) => model.update(ctx, |model, ctx| {
                model.discard_files(file_infos, should_stash, branch_name, ctx)
            }),
            Self::Remote(model) => model.update(ctx, |model, ctx| {
                model.discard_files(file_infos, should_stash, branch_name, ctx)
            }),
        }
    }

    #[cfg(feature = "local_fs")]
    pub fn stop_active_watcher(&self, ctx: &mut ModelContext<Self>) {
        if let Self::Local(model) = self {
            model.update(ctx, |model, ctx| model.stop_active_watcher(ctx));
        }
    }

    pub async fn load_diff_data_for_mode(
        mode: DiffMode,
        repo_path: PathBuf,
    ) -> Option<GitDiffData> {
        LocalDiffStateModel::load_diff_data_for_mode(mode, repo_path).await
    }

    pub async fn get_all_branches(
        repo_path: &Path,
        max_branch_count: Option<usize>,
        include_remotes: bool,
    ) -> Result<Vec<(String, bool)>> {
        LocalDiffStateModel::get_all_branches(repo_path, max_branch_count, include_remotes).await
    }

    pub async fn get_all_branches_with_known_main(
        repo_path: &Path,
        main_branch: &str,
        max_branch_count: Option<usize>,
        include_remotes: bool,
    ) -> Result<Vec<(String, bool)>> {
        LocalDiffStateModel::get_all_branches_with_known_main(
            repo_path,
            main_branch,
            max_branch_count,
            include_remotes,
        )
        .await
    }

    pub fn sort_branches_main_first(
        branches: &[(String, bool)],
    ) -> impl Iterator<Item = &(String, bool)> {
        LocalDiffStateModel::sort_branches_main_first(branches)
    }

    pub(crate) fn parse_unified_diff_header(header_line: &str) -> Result<UnifiedDiffHeader> {
        LocalDiffStateModel::parse_unified_diff_header(header_line)
    }

    pub(crate) async fn compute_merge_base(repo_path: &Path, mode: &DiffMode) -> Result<String> {
        LocalDiffStateModel::compute_merge_base(repo_path, mode).await
    }

    pub(crate) async fn retrieve_diff_state(
        repo_path: &Path,
        file: &Path,
        mode: &DiffMode,
        merge_base: Option<&str>,
    ) -> Result<(String, Option<Arc<FileDiffAndContent>>)> {
        LocalDiffStateModel::retrieve_diff_state(repo_path, file, mode, merge_base).await
    }

    pub(crate) async fn diff_metadata_against_head(
        repo_path: &Path,
    ) -> Result<DiffMetadataAgainstBase> {
        LocalDiffStateModel::diff_metadata_against_head(repo_path).await
    }

    #[cfg(test)]
    pub fn new_for_test(ctx: &mut ModelContext<Self>) -> Self {
        let local = ctx.add_model(|ctx| LocalDiffStateModel::new(None, ctx));
        ctx.subscribe_to_model(&local, Self::forward_event);
        Self::Local(local)
    }
}
