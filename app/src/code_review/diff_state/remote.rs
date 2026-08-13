use std::path::{Path, PathBuf};

use anyhow::Result;
use warpui::{AppContext, ModelContext};

use super::{
    Commit, DiffMetadataAgainstBase, DiffMode, DiffState, DiffStateModelEvent, DiffStats,
    FileDiffAndContent, FileStatusInfo, GitDiffData, PrInfo,
};

pub struct RemoteDiffStateModel {
    remote_path: repo_metadata::RemoteRepositoryIdentifier,
    mode: DiffMode,
}

impl RemoteDiffStateModel {
    pub fn new(
        remote_path: repo_metadata::RemoteRepositoryIdentifier,
        _ctx: &mut ModelContext<Self>,
    ) -> Self {
        Self {
            remote_path,
            mode: DiffMode::default(),
        }
    }

    pub fn get(&self) -> DiffState {
        DiffState::NotInRepository
    }

    pub fn diff_mode(&self) -> DiffMode {
        self.mode.clone()
    }

    pub fn get_uncommitted_stats(&self) -> Option<DiffStats> {
        None
    }

    pub fn get_main_branch_stats(&self) -> Option<DiffStats> {
        None
    }

    pub fn get_diff_stats(&self) -> super::GitDiffInfo {
        super::GitDiffInfo {
            uncommitted_stats: None,
            main_branch_stats: None,
            main_branch_name: None,
        }
    }

    pub fn get_stats_for_current_mode(&self) -> Option<DiffStats> {
        None
    }

    pub fn get_stats_for_mode(&self, _mode: DiffMode) -> Option<DiffStats> {
        None
    }

    pub fn diff_mode_for_base_branch(&self, base_branch: Option<&str>) -> DiffMode {
        DiffMode::from_branch(base_branch.unwrap_or_default(), None)
    }

    pub fn get_current_branch_name(&self) -> Option<String> {
        None
    }

    pub fn get_main_branch_name(&self) -> Option<String> {
        None
    }

    pub fn is_on_main_branch(&self) -> bool {
        false
    }

    pub fn unpushed_commits(&self) -> &[Commit] {
        &[]
    }

    pub fn upstream_ref(&self) -> Option<&str> {
        None
    }

    pub fn upstream_differs_from_main(&self) -> bool {
        false
    }

    pub fn pr_info(&self) -> Option<&PrInfo> {
        None
    }

    pub fn is_pr_info_refreshing(&self) -> bool {
        false
    }

    pub fn is_git_operation_blocked(&self, _ctx: &AppContext) -> bool {
        false
    }

    pub fn has_head(&self) -> bool {
        false
    }

    pub fn active_repository_path(&self, _ctx: &AppContext) -> Option<PathBuf> {
        None
    }

    pub fn is_inside_repository(&self) -> bool {
        true
    }

    pub fn set_diff_mode(
        &mut self,
        mode: DiffMode,
        _should_fetch_base: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        self.mode = mode;
        ctx.emit(DiffStateModelEvent::DiffModeChanged {
            should_fetch_base: false,
        });
    }

    pub fn set_diff_mode_and_fetch_base(&mut self, mode: DiffMode, ctx: &mut ModelContext<Self>) {
        self.set_diff_mode(mode, true, ctx);
    }

    pub fn load_diffs_for_current_repo(
        &mut self,
        _should_fetch_base: bool,
        _ctx: &mut ModelContext<Self>,
    ) {
    }

    pub fn set_code_review_metadata_refresh_enabled(
        &mut self,
        _enabled: bool,
        _ctx: &mut ModelContext<Self>,
    ) {
    }

    pub fn refresh_metadata_and_pr_info(&mut self, _ctx: &mut ModelContext<Self>) {}

    pub fn refresh_diff_metadata_for_current_repo(
        &mut self,
        _invalidation_behavior: super::InvalidationBehavior,
        _ctx: &mut ModelContext<Self>,
    ) {
    }

    pub fn refresh_pr_info(&mut self, _ctx: &mut ModelContext<Self>) {}

    pub fn discard_files(
        &mut self,
        _file_infos: Vec<FileStatusInfo>,
        _should_stash: bool,
        _branch_name: Option<String>,
        _ctx: &mut ModelContext<Self>,
    ) {
    }

    pub fn get_diff_data_for_mode(
        &self,
        _mode: DiffMode,
        _repo_path: PathBuf,
    ) -> impl std::future::Future<Output = Option<GitDiffData>> {
        std::future::ready(None)
    }
}

impl warpui::Entity for RemoteDiffStateModel {
    type Event = DiffStateModelEvent;
}

#[allow(dead_code)]
fn _remote_path(_model: &RemoteDiffStateModel) -> &repo_metadata::RemoteRepositoryIdentifier {
    &_model.remote_path
}

#[allow(dead_code)]
async fn _remote_file_diff(_path: &Path) -> Result<Option<FileDiffAndContent>> {
    Ok(None)
}
