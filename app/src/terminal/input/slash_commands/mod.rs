mod data_source;
mod search_item;
pub(super) mod view;

pub use data_source::*;
pub use view::{CloseReason, InlineSlashCommandView, SlashCommandsEvent};

#[cfg(feature = "local_fs")]
use std::path::PathBuf;

use warp_core::ui::theme::AnsiColorIdentifier;
#[cfg(feature = "local_fs")]
use warp_util::path::{CleanPathResult, LineAndColumnArg};
use warpui::clipboard::ClipboardContent;
use warpui::{SingletonEntity, ViewContext};

use crate::ai::blocklist::agent_view::AgentViewEntryOrigin;
use crate::ai::blocklist::{
    BlocklistAIHistoryModel, InputConfig, InputType, QueuedQuery, QueuedQueryModel,
    QueuedQueryOrigin,
};
use crate::cloud_object::model::persistence::CloudModel;
use crate::code_review::events::CodeReviewPaneEntrypoint;
use crate::object_ids::SyncId;
use crate::search::slash_command_menu::static_commands::commands::{self, COMMAND_REGISTRY};
use crate::search::slash_command_menu::static_commands::Availability;
use crate::search::slash_command_menu::{SlashCommandId, StaticCommand};
use crate::settings::AISettings;
use crate::tab::SelectedTabColor;
use crate::terminal::input::decorations::InputBackgroundJobOptions;
use crate::terminal::input::inline_menu::{InlineMenuAction, InlineMenuType};
use crate::terminal::input::slash_command_model::{
    SlashCommandEntryState, UpdatedSlashCommandModel,
};
use crate::terminal::input::{
    CompletionsTrigger, Event, Input, InputSuggestionsMode, UserQueryMenuAction,
};
#[cfg(feature = "local_fs")]
use crate::terminal::model::session::Session;
use crate::terminal::view::TerminalAction;
use crate::ui_components::color_dot;
use crate::view_components::DismissibleToast;
use crate::workflows::{WorkflowSelectionSource, WorkflowSource, WorkflowType};
use crate::workspace::{ForkedConversationDestination, ToastStack, WorkspaceAction};

#[derive(Debug, Clone)]
pub enum AcceptSlashCommandOrSavedPrompt {
    SlashCommand {
        id: SlashCommandId,
    },
    AcpCommand {
        name: String,
        description: String,
        input_hint: Option<String>,
    },
    SavedPrompt {
        id: SyncId,
    },
}
impl InlineMenuAction for AcceptSlashCommandOrSavedPrompt {
    const MENU_TYPE: InlineMenuType = InlineMenuType::SlashCommands;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SlashCommandTrigger {
    Input { cmd_or_ctrl_enter: bool },
    Keybinding,
}

impl SlashCommandTrigger {
    fn cmd_or_ctrl_enter() -> Self {
        Self::Input {
            cmd_or_ctrl_enter: true,
        }
    }

    pub fn input() -> Self {
        Self::Input {
            cmd_or_ctrl_enter: false,
        }
    }

    pub(super) fn keybinding() -> Self {
        Self::Keybinding
    }

    pub fn is_keybinding(&self) -> bool {
        matches!(self, Self::Keybinding)
    }

    fn is_cmd_or_ctrl_enter(&self) -> bool {
        matches!(
            self,
            Self::Input {
                cmd_or_ctrl_enter: true
            }
        )
    }
}

#[cfg(feature = "local_fs")]
fn open_file_command_path(
    session: &Session,
    current_dir: &str,
    raw_arg: &str,
) -> (PathBuf, Option<LineAndColumnArg>) {
    let parsed_path = CleanPathResult::with_line_and_column_number(raw_arg.trim());
    // The argument may contain shell-escaped characters (e.g. `\ ` for spaces) from auto-suggest.
    // Unescape them so the path matches the actual filesystem entry.
    let unescaped_path = session.shell_family().unescape(&parsed_path.path);
    // Expand `~` to the user's home directory.
    let expanded_path = shellexpand::tilde(&unescaped_path);

    let shell_path = session
        .convert_directory_to_typed_path_buf(current_dir.to_owned())
        .join(session.convert_directory_to_typed_path_buf(expanded_path.into_owned()))
        .normalize();
    let file_path = session
        .maybe_convert_to_native_path(&shell_path.to_path())
        .unwrap_or_else(|err| {
            log::warn!("unable to convert /open-file path to native path: {err:?}");
            PathBuf::from(shell_path.to_string_lossy().into_owned())
        });

    (file_path, parsed_path.line_and_column_num)
}

impl Input {
    pub(super) fn select_slash_command(
        &mut self,
        command: &StaticCommand,
        trigger: SlashCommandTrigger,
        ctx: &mut ViewContext<Self>,
    ) {
        if command.argument.as_ref().is_none() {
            self.execute_slash_command(
                command, None, trigger, /*is_queued_prompt*/ false, ctx,
            );
        } else if command
            .argument
            .as_ref()
            .is_some_and(|arg| arg.should_execute_on_selection)
        {
            // TODO (zachbai): caller
            // should probably be invoking `execute_slash_command` in this case.
            let argument = if !self.suggestions_mode_model.as_ref(ctx).is_slash_commands() {
                let trimmed = self.buffer_text(ctx).trim().to_owned();
                (!trimmed.is_empty()).then_some(trimmed)
            } else {
                None
            };
            self.execute_slash_command(
                command,
                argument.as_ref(),
                trigger,
                /*is_queued_prompt*/ false,
                ctx,
            );
        } else {
            if command.auto_enter_ai_mode {
                self.enter_auto_slash_command_ai_mode(trigger, ctx);
            }
            self.editor.update(ctx, |editor, ctx| {
                editor.set_buffer_text(&format!("{} ", command.name), ctx);
            });
        }
    }

    fn enter_auto_slash_command_ai_mode(
        &mut self,
        trigger: SlashCommandTrigger,
        ctx: &mut ViewContext<Self>,
    ) {
        if !self.agent_view_controller.as_ref(ctx).is_active() {
            self.ai_context_model.update(ctx, |context_model, ctx| {
                context_model.set_pending_query_state_for_new_conversation(
                    AgentViewEntryOrigin::SlashCommand { trigger },
                    ctx,
                );
            });
        }

        let is_input_buffer_empty = self.editor.as_ref(ctx).buffer_text(ctx).is_empty();
        self.ai_input_model.update(ctx, |input_model, ctx| {
            input_model.set_input_config(
                InputConfig {
                    input_type: InputType::AI,
                    is_locked: true,
                },
                is_input_buffer_empty,
                ctx,
            );
        });
    }

    pub(super) fn close_slash_commands_menu(&mut self, ctx: &mut ViewContext<Self>) {
        self.suggestions_mode_model.update(ctx, |model, ctx| {
            model.set_mode(InputSuggestionsMode::Closed, ctx);
        });
        ctx.notify();
    }

    pub(super) fn handle_slash_command_model_event(
        &mut self,
        event: &UpdatedSlashCommandModel,
        ctx: &mut ViewContext<Self>,
    ) {
        // Refresh decorations if the slash command detection state changed, since
        // detected commands affect syntax highlighting.
        let new_state = self.slash_command_model.as_ref(ctx).state();
        if event.old_state.is_detected_command() != new_state.is_detected_command() {
            let _ = self
                .debounce_input_background_tx
                .try_send(InputBackgroundJobOptions::default().with_command_decoration());
        }

        match self.slash_command_model.as_ref(ctx).state().clone() {
            SlashCommandEntryState::None | SlashCommandEntryState::DisabledUntilEmptyBuffer => {
                if self.suggestions_mode_model.as_ref(ctx).is_slash_commands() {
                    self.close_slash_commands_menu(ctx);
                }
            }
            SlashCommandEntryState::Composing { .. } => {
                if self.suggestions_mode_model.as_ref(ctx).is_closed() {
                    self.open_slash_commands_menu(ctx);
                } else if !self.suggestions_mode_model.as_ref(ctx).is_slash_commands() {
                    self.slash_command_model.update(ctx, |model, ctx| {
                        model.disable(ctx);
                    });
                }
            }
            SlashCommandEntryState::SlashCommand(detected_command) => {
                // If there is only one result (or zero, but that should be impossible if there is
                // a valid command in the input) OR if the user has started typing arguments, hide
                // the menu.
                if self.suggestions_mode_model.as_ref(ctx).is_slash_commands()
                    && (self
                        .inline_slash_commands_view
                        .as_ref(ctx)
                        .result_count(ctx)
                        < 2
                        || detected_command.argument.is_some())
                {
                    self.close_slash_commands_menu(ctx);
                }

                if detected_command.command.auto_enter_ai_mode {
                    self.enter_auto_slash_command_ai_mode(SlashCommandTrigger::input(), ctx);
                }

                if detected_command.command.name == commands::EDIT.name
                    && detected_command
                        .argument
                        .as_ref()
                        .is_some_and(|argument| argument.is_empty())
                    && self.suggestions_mode_model.as_ref(ctx).is_closed()
                {
                    self.open_completion_suggestions(CompletionsTrigger::SlashCommandAutoOpen, ctx);
                }
            }
        }
    }

    pub(crate) fn handle_slash_commands_menu_event(
        &mut self,
        event: &SlashCommandsEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            SlashCommandsEvent::Close(reason) => {
                if reason.is_manual_dismissal() {
                    self.slash_command_model.update(ctx, |model, ctx| {
                        model.disable(ctx);
                    });
                }

                self.suggestions_mode_model.update(ctx, |model, ctx| {
                    model.set_mode(InputSuggestionsMode::Closed, ctx);
                });
                ctx.notify();
            }
            SlashCommandsEvent::SelectedSavedPrompt { id } => {
                let Some(workflow) = CloudModel::as_ref(ctx).get_workflow(id).cloned() else {
                    log::warn!("Tried to execute workflow for id {id:?} but it does not exist");
                    return;
                };
                self.show_workflows_info_box_on_workflow_selection(
                    WorkflowType::Saved(Box::new(workflow)),
                    WorkflowSource::Agent,
                    WorkflowSelectionSource::SlashMenu,
                    None,
                    ctx,
                );
            }
            SlashCommandsEvent::SelectedStaticCommand {
                id,
                cmd_or_ctrl_enter,
            } => {
                let Some(command) = COMMAND_REGISTRY.get_command(id) else {
                    return;
                };
                self.select_slash_command(
                    command,
                    SlashCommandTrigger::Input {
                        cmd_or_ctrl_enter: *cmd_or_ctrl_enter,
                    },
                    ctx,
                );
            }
            SlashCommandsEvent::SelectedAcpCommand {
                name, input_hint, ..
            } => {
                let prompt = format!("/{name}");
                if input_hint.is_some() {
                    self.enter_auto_slash_command_ai_mode(SlashCommandTrigger::input(), ctx);
                    self.editor.update(ctx, |editor, ctx| {
                        editor.set_buffer_text(&format!("{prompt} "), ctx);
                    });
                    self.close_slash_commands_menu(ctx);
                } else {
                    self.editor.update(ctx, |editor, ctx| {
                        editor.set_buffer_text(&prompt, ctx);
                    });
                    self.close_slash_commands_menu(ctx);
                    self.submit_ai_query(None, ctx);
                }
            }
        }
    }

    /// Executes the given `command` with `argument`, if any.
    ///
    /// When `is_queued_prompt` is true, this is the first send of a previously queued prompt:
    /// the input buffer is left alone so the user doesn't lose anything they've typed while
    /// the agent was busy.
    ///
    /// Returns `true` if execution was 'handled' (whether or not it resulted in success or failure).
    pub(super) fn execute_slash_command(
        &mut self,
        command: &StaticCommand,
        argument: Option<&String>,
        trigger: SlashCommandTrigger,
        is_queued_prompt: bool,
        ctx: &mut ViewContext<Self>,
    ) -> bool {
        fn show_error_toast(message: String, ctx: &mut ViewContext<Input>) {
            let window_id = ctx.window_id();
            ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                toast_stack.add_ephemeral_toast(DismissibleToast::error(message), window_id, ctx);
            });
        }

        if command.availability.contains(Availability::AI_ENABLED)
            && !AISettings::as_ref(ctx).is_any_ai_enabled(ctx)
        {
            show_error_toast(format!("{} requires AI to be enabled", command.name), ctx);
            return true;
        }

        // Handle the slash command action based on its kind
        match command.name {
            agent if command.name == commands::AGENT.name || command.name == commands::NEW.name => {
                let prompt = argument.and_then(|argument| {
                    let trimmed = argument.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_owned())
                    }
                });
                if prompt.is_some() {
                    self.ai_input_model.update(ctx, |model, ctx| {
                        model.handle_input_buffer_submitted(ctx);
                    });
                }
                ctx.emit(Event::EnterAgentView {
                    initial_prompt: prompt,
                    conversation_id: None,
                    origin: AgentViewEntryOrigin::SlashCommand { trigger },
                });
            }
            add_prompt if command.name == commands::ADD_PROMPT.name => {
                ctx.dispatch_typed_action(&TerminalAction::OpenAddPromptPane);
            }
            add_rule if command.name == commands::ADD_RULE.name => {
                ctx.dispatch_typed_action(&TerminalAction::OpenAddRulePane);
            }
            create_docker_sandbox if command.name == commands::CREATE_DOCKER_SANDBOX.name => {
                ctx.emit(Event::CreateDockerSandbox);
            }
            conversations if command.name == commands::CONVERSATIONS.name => {
                self.open_conversation_menu(ctx);
            }
            rename_tab if command.name == commands::RENAME_TAB.name => {
                let Some(name) = argument
                    .map(|name| name.trim())
                    .filter(|name| !name.is_empty())
                else {
                    show_error_toast(
                        "Please provide a tab name after /rename-tab".to_owned(),
                        ctx,
                    );
                    return true;
                };

                ctx.dispatch_typed_action(&WorkspaceAction::SetActiveTabName(name.to_owned()));
            }
            set_tab_color if command.name == commands::SET_TAB_COLOR.name => {
                let supported_options = || {
                    color_dot::TAB_COLOR_OPTIONS
                        .iter()
                        .map(|c| c.to_string().to_ascii_lowercase())
                        .chain(std::iter::once("none".to_owned()))
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                let Some(arg) = argument
                    .map(|name| name.trim())
                    .filter(|name| !name.is_empty())
                else {
                    show_error_toast(
                        format!(
                            "Please provide a color after /set-tab-color ({})",
                            supported_options()
                        ),
                        ctx,
                    );
                    return true;
                };

                let color = if arg.eq_ignore_ascii_case("none") {
                    SelectedTabColor::Cleared
                } else {
                    let parsed = arg
                        .parse::<AnsiColorIdentifier>()
                        .ok()
                        .filter(|c| color_dot::TAB_COLOR_OPTIONS.contains(c));
                    match parsed {
                        Some(c) => SelectedTabColor::Color(c),
                        None => {
                            show_error_toast(
                                format!(
                                    "Unknown tab color '{arg}'. Use one of: {}.",
                                    supported_options()
                                ),
                                ctx,
                            );
                            return true;
                        }
                    }
                };

                ctx.dispatch_typed_action(&WorkspaceAction::SetActiveTabColor(color));
            }
            edit if command.name == commands::EDIT.name => {
                #[cfg(feature = "local_fs")]
                match argument {
                    Some(args) if !args.is_empty() => {
                        let Some(session_id) = self.active_block_session_id() else {
                            return false;
                        };

                        let Some(session) = self.sessions.as_ref(ctx).get(session_id) else {
                            return false;
                        };

                        if !session.is_local() {
                            let window_id = ctx.window_id();
                            ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                                toast_stack.add_ephemeral_toast(
                                    DismissibleToast::error(
                                        "The /open-file command is only available for local sessions"
                                            .to_owned(),
                                    ),
                                    window_id,
                                    ctx,
                                );
                            });
                            return false;
                        }

                        let current_dir = self
                            .active_block_metadata
                            .as_ref()
                            .and_then(|metadata| metadata.current_working_directory())
                            .map(str::to_owned);

                        let Some(current_dir) = current_dir else {
                            return false;
                        };

                        let (file_path, line_col) =
                            open_file_command_path(&session, &current_dir, args);

                        match std::fs::metadata(&file_path) {
                            Ok(metadata) if metadata.is_file() => {
                                use crate::util::file::external_editor;

                                ctx.dispatch_typed_action(&TerminalAction::OpenCodeInWarp {
                                    path: file_path,
                                    layout: external_editor::settings::EditorLayout::SplitPane,
                                    line_col,
                                });
                            }
                            Ok(_) => {
                                show_error_toast(
                                    "The /open-file command only works for files, not directories"
                                        .to_owned(),
                                    ctx,
                                );
                                return true;
                            }
                            Err(_) => {
                                show_error_toast(
                                    format!("File not found: {}", file_path.display()),
                                    ctx,
                                );
                                return true;
                            }
                        }
                    }
                    _ => {
                        use crate::ui_events::PaletteSource;

                        ctx.emit(Event::OpenFilesPalette {
                            source: PaletteSource::Keybinding,
                        });
                    }
                }
                #[cfg(not(feature = "local_fs"))]
                {
                    show_error_toast(
                        "The /open-file command is not supported in this build".to_owned(),
                        ctx,
                    );
                    return true;
                }
            }
            export_to_clipboard if command.name == commands::EXPORT_TO_CLIPBOARD.name => {
                let history = BlocklistAIHistoryModel::handle(ctx);
                let Some(conversation) = history
                    .as_ref(ctx)
                    .active_conversation(self.terminal_view_id)
                else {
                    show_error_toast("No active conversation to export".to_owned(), ctx);
                    return true;
                };

                let action_model = self.ai_action_model.as_ref(ctx);
                let conversation_text = conversation.export_to_markdown(Some(action_model));

                ctx.clipboard()
                    .write(ClipboardContent::plain_text(conversation_text));

                // Show a toast to confirm the export
                let window_id = ctx.window_id();
                ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    let toast = DismissibleToast::default(String::from(
                        "Conversation exported to clipboard",
                    ));
                    toast_stack.add_ephemeral_toast(toast, window_id, ctx);
                });
            }
            export_to_file if command.name == commands::EXPORT_TO_FILE.name => {
                self.export_conversation_to_file(argument.map(|filename| filename.to_owned()), ctx);
            }
            open_code_review if command.name == commands::OPEN_CODE_REVIEW.name => {
                ctx.dispatch_typed_action(&TerminalAction::ToggleCodeReviewPane {
                    entrypoint: CodeReviewPaneEntrypoint::SlashCommand,
                });
            }
            open_settings_file if command.name == commands::OPEN_SETTINGS_FILE.name => {
                if !cfg!(feature = "local_fs") {
                    return false;
                }
                ctx.dispatch_typed_action(&WorkspaceAction::OpenSettingsFile);
            }
            open_project_rules if command.name == commands::OPEN_PROJECT_RULES.name => {
                ctx.dispatch_typed_action(&TerminalAction::OpenProjectRulesPane);
            }
            open_rules if command.name == commands::OPEN_RULES.name => {
                ctx.dispatch_typed_action(&TerminalAction::OpenRulesPane);
            }
            prompts if command.name == commands::PROMPTS.name => {
                self.open_prompts_menu(ctx);
            }
            rewind if command.name == commands::REWIND.name => {
                self.open_rewind_menu(ctx);
            }
            fork if command.name == commands::FORK.name => {
                let Some(conversation_id) = self
                    .ai_context_model
                    .as_ref(ctx)
                    .selected_conversation_id(ctx)
                else {
                    show_error_toast("/fork requires an active conversation".to_owned(), ctx);
                    return true;
                };

                let destination = if trigger.is_cmd_or_ctrl_enter() {
                    ForkedConversationDestination::NewTab
                } else {
                    ForkedConversationDestination::SplitPane
                };

                ctx.dispatch_typed_action(&WorkspaceAction::ForkAIConversation {
                    conversation_id,
                    fork_from_exchange: None,
                    initial_prompt: argument.cloned(),
                    destination,
                });
            }
            fork_from if command.name == commands::FORK_FROM.name => {
                self.open_user_query_menu(UserQueryMenuAction::ForkFrom, ctx);
                return true;
            }
            queue if command.name == commands::QUEUE.name => {
                let Some(conversation_id) = self
                    .ai_context_model
                    .as_ref(ctx)
                    .selected_conversation_id(ctx)
                else {
                    show_error_toast("/queue requires an active conversation".to_owned(), ctx);
                    return true;
                };

                let Some(prompt) = argument.filter(|a| !a.is_empty()).cloned() else {
                    show_error_toast("/queue requires a prompt argument".to_owned(), ctx);
                    return true;
                };

                let history = BlocklistAIHistoryModel::handle(ctx);
                let is_in_progress = history
                    .as_ref(ctx)
                    .conversation(&conversation_id)
                    .is_some_and(|c| c.status().is_in_progress() || c.status().is_blocked());

                if is_in_progress {
                    let attachments = self.ai_context_model.update(ctx, |context_model, ctx| {
                        context_model.take_pending_attachments(ctx)
                    });
                    QueuedQueryModel::handle(ctx).update(ctx, |model, ctx| {
                        model.append(
                            conversation_id,
                            QueuedQuery::new_with_attachments(
                                prompt,
                                QueuedQueryOrigin::QueueSlashCommand,
                                attachments,
                            ),
                            ctx,
                        );
                    });
                } else {
                    self.submit_user_query_now(prompt, ctx);
                }
            }
            open_repo if command.name == commands::OPEN_REPO.name => {
                self.open_repos_menu(ctx);
            }
            _ => {
                debug_assert!(
                    false,
                    "Attempted to execute slash command with no handler: {}",
                    command.name
                );
                return false;
            }
        }

        // Leave the buffer alone when re-sending a queued prompt (the user may have typed
        // new input while the agent was busy).
        if !is_queued_prompt {
            self.editor.update(ctx, |editor, ctx| {
                editor.clear_buffer(ctx);
            });
        }

        true
    }

    /// Handles cmd+enter (Mac) / ctrl+enter (Linux/Windows) for slash commands.
    ///
    /// Returns `true` if the keypress was handled.
    pub(super) fn maybe_handle_cmd_or_ctrl_shift_enter_for_slash_command(
        &mut self,
        ctx: &mut ViewContext<Self>,
    ) -> bool {
        // If slash command menu is open, accept the selected item with cmd_or_ctrl_enter=true.
        if matches!(
            self.suggestions_mode_model.as_ref(ctx).mode(),
            InputSuggestionsMode::SlashCommands
        ) {
            self.inline_slash_commands_view.update(ctx, |view, ctx| {
                view.accept_selected_item(true, ctx);
            });
            return true;
        }

        // If no menu but slash command detected in buffer, execute with cmd_or_ctrl_enter=true
        match self.slash_command_model.as_ref(ctx).state() {
            SlashCommandEntryState::SlashCommand(detected_command) => {
                let command = detected_command.command.clone();
                let argument = detected_command.argument.clone();
                self.execute_slash_command(
                    &command,
                    argument.as_ref(),
                    SlashCommandTrigger::cmd_or_ctrl_enter(),
                    /*is_queued_prompt*/ false,
                    ctx,
                )
            }
            SlashCommandEntryState::None
            | SlashCommandEntryState::Composing { .. }
            | SlashCommandEntryState::DisabledUntilEmptyBuffer => false,
        }
    }

    /// Executes a slash command on `enter` keypress.
    ///
    /// If the slash command menu is open, then "accepts" the slash command:
    ///   * If the slash command does not take arguments, executes it
    ///   * If the slash command does take arguments, inserts it into the input.
    ///
    /// If the slash command menu is not open, then "executes" the slash command in the input, if
    /// there is one.
    ///
    /// Returns `true` if the enter keypress was 'handled', else upstream enter keypress handling
    /// logic should continue.
    pub(super) fn maybe_handle_enter_for_slash_command(
        &mut self,
        ctx: &mut ViewContext<Self>,
    ) -> bool {
        let buffer_text = self.editor.as_ref(ctx).buffer_text(ctx);
        let detected = self
            .slash_command_model
            .as_ref(ctx)
            .detect_command(&buffer_text, ctx);

        match detected {
            SlashCommandEntryState::SlashCommand(detected_command) => {
                let command = detected_command.command.clone();
                let argument = detected_command.argument.clone();
                self.execute_slash_command(
                    &command,
                    argument.as_ref(),
                    SlashCommandTrigger::input(),
                    /*is_queued_prompt*/ false,
                    ctx,
                )
            }
            SlashCommandEntryState::None
            | SlashCommandEntryState::Composing { .. }
            | SlashCommandEntryState::DisabledUntilEmptyBuffer => {
                if matches!(
                    self.suggestions_mode_model.as_ref(ctx).mode(),
                    InputSuggestionsMode::SlashCommands
                ) {
                    self.inline_slash_commands_view.update(ctx, |view, ctx| {
                        view.accept_selected_item(false, ctx);
                    });
                    return true;
                }

                false
            }
        }
    }
}
