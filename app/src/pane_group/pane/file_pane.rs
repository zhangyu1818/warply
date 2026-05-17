use std::{path::PathBuf, sync::Arc};

use warpui::{AppContext, ModelHandle, SingletonEntity, View, ViewContext, ViewHandle};

use crate::code::editor_management::CodeSource;
use crate::{
    app_state::{CodePaneSnapShot, CodePaneTabSnapshot, LeafContents},
    notebooks::file::{FileNotebookEvent, FileNotebookView},
    terminal::model::session::Session,
    workspace::ActiveSession,
};

use super::{view::PaneView, DetachType, PaneConfiguration, PaneContent, PaneGroup, PaneId};

pub struct FilePane {
    view: ViewHandle<PaneView<FileNotebookView>>,
    pane_configuration: ModelHandle<PaneConfiguration>,
}

impl FilePane {
    fn from_view(file_view: ViewHandle<FileNotebookView>, ctx: &mut AppContext) -> Self {
        let pane_configuration = file_view.as_ref(ctx).pane_configuration();

        let view = ctx.add_typed_action_view(file_view.window_id(ctx), |ctx| {
            let pane_id = PaneId::from_file_pane_ctx(ctx);
            PaneView::new(pane_id, file_view, (), pane_configuration.clone(), ctx)
        });

        Self {
            view,
            pane_configuration,
        }
    }

    pub fn new<V: View>(
        path: Option<PathBuf>,
        target_session: Option<Arc<Session>>,
        #[cfg(feature = "local_fs")] code_source: Option<CodeSource>,
        ctx: &mut ViewContext<V>,
    ) -> Self {
        let view = ctx.add_typed_action_view(move |ctx| {
            let mut view = FileNotebookView::new(ctx);
            #[cfg(feature = "local_fs")]
            view.set_code_source(code_source);

            if let Some(path) = path {
                if let Some(target_session) = target_session {
                    if target_session.is_local() {
                        view.open_local(path, Some(target_session), ctx);
                    }
                } else {
                    let session = ActiveSession::as_ref(ctx)
                        .session(ctx.window_id())
                        .filter(|session| session.is_local());
                    view.open_local(path, session, ctx);
                }
            }

            view
        });
        Self::from_view(view, ctx)
    }

    pub fn file_view(&self, ctx: &AppContext) -> ViewHandle<FileNotebookView> {
        self.view.as_ref(ctx).child(ctx)
    }
}

impl PaneContent for FilePane {
    fn id(&self) -> PaneId {
        PaneId::from_file_pane_view(&self.view)
    }

    fn attach(
        &self,
        _group: &PaneGroup,
        focus_handle: crate::pane_group::focus_state::PaneFocusHandle,
        ctx: &mut ViewContext<PaneGroup>,
    ) {
        self.view
            .update(ctx, |view, ctx| view.set_focus_handle(focus_handle, ctx));

        let pane_id = self.id();

        ctx.subscribe_to_view(
            &self.file_view(ctx),
            move |pane_group, _, event, ctx| match event {
                FileNotebookEvent::FileLoaded => {
                    ctx.emit(crate::pane_group::Event::AppStateChanged)
                }
                #[cfg(feature = "local_fs")]
                FileNotebookEvent::OpenFileWithTarget {
                    path,
                    target,
                    line_col,
                } => {
                    ctx.emit(crate::pane_group::Event::OpenFileWithTarget {
                        path: path.clone(),
                        target: target.clone(),
                        line_col: *line_col,
                    });
                }
                FileNotebookEvent::Pane(pane_event) => {
                    pane_group.handle_pane_event(pane_id, pane_event, ctx)
                }
            },
        );

        ctx.subscribe_to_view(&self.view, move |group, _, event, ctx| {
            group.handle_pane_view_event(pane_id, event, ctx);
        });
    }

    fn detach(
        &self,
        _group: &PaneGroup,
        _detach_type: DetachType,
        ctx: &mut ViewContext<PaneGroup>,
    ) {
        let file_view = self.file_view(ctx);
        ctx.unsubscribe_to_view(&file_view);
        ctx.unsubscribe_to_view(&self.view);
    }

    fn snapshot(&self, app: &AppContext) -> LeafContents {
        match self.file_view(app).as_ref(app).local_path() {
            Some(path) => LeafContents::Code(CodePaneSnapShot::Local {
                tabs: vec![CodePaneTabSnapshot {
                    path: Some(path.clone()),
                }],
                active_tab_index: 0,
                source: Some(CodeSource::Link {
                    path,
                    range_start: None,
                    range_end: None,
                }),
            }),
            None => LeafContents::Welcome {
                startup_directory: None,
            },
        }
    }

    fn has_application_focus(&self, ctx: &mut ViewContext<PaneGroup>) -> bool {
        self.view.is_self_or_child_focused(ctx)
    }

    fn focus(&self, ctx: &mut ViewContext<PaneGroup>) {
        self.file_view(ctx).update(ctx, |view, ctx| view.focus(ctx));
    }

    fn pane_configuration(&self) -> ModelHandle<PaneConfiguration> {
        self.pane_configuration.clone()
    }

    fn is_pane_being_dragged(&self, ctx: &AppContext) -> bool {
        self.view.as_ref(ctx).is_being_dragged()
    }
}
