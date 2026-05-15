use std::path::PathBuf;

use warp_util::path::EscapeChar;
use warpui::{App, EntityId, ModelHandle};

use crate::ai::active_agent_views_model::ActiveAgentViewsModel;
use crate::terminal::cli_agent_sessions::CLIAgentSessionsModel;
use crate::{
    ai::{
        agent::conversation::AIConversationId,
        blocklist::permissions::{
            CommandExecutionPermission, CommandExecutionPermissionAllowedReason,
            CommandExecutionPermissionDeniedReason, FileReadPermission,
            FileReadPermissionAllowedReason, FileReadPermissionDeniedReason, FileWritePermission,
            FileWritePermissionAllowedReason, FileWritePermissionDeniedReason,
        },
        execution_profiles::{
            profiles::AIExecutionProfilesModel, ActionPermission, WriteToPtyPermission,
        },
    },
    cloud_object::model::persistence::CloudModel,
    cloud_object::update_manager::UpdateManager,
    identity::LocalIdentityProvider,
    settings::AgentModeCommandExecutionPredicate,
    test_util::settings::initialize_settings_for_tests,
    GlobalResourceHandles, GlobalResourceHandlesProvider, LaunchMode,
};

use super::{BlocklistAIHistoryModel, BlocklistAIPermissions};

struct PermissionsTestState {
    convo_id: AIConversationId,
    permissions: ModelHandle<BlocklistAIPermissions>,
    history: ModelHandle<BlocklistAIHistoryModel>,
    terminal_view_id: EntityId,
    profile_model: ModelHandle<AIExecutionProfilesModel>,
}

fn initialize_permissions_test(app: &mut App) -> PermissionsTestState {
    initialize_settings_for_tests(app);
    let global_resource_handles = GlobalResourceHandles::mock(app);
    app.add_singleton_model(|_| GlobalResourceHandlesProvider::new(global_resource_handles));
    let history = app.add_singleton_model(|_| BlocklistAIHistoryModel::new(vec![], &[]));
    app.add_singleton_model(|_| CLIAgentSessionsModel::new());
    app.add_singleton_model(|_| ActiveAgentViewsModel::new());
    let permissions = app.add_singleton_model(BlocklistAIPermissions::new);
    let terminal_view_id = EntityId::new();
    app.add_singleton_model(|_| LocalIdentityProvider::new_for_test());
    app.add_singleton_model(UpdateManager::mock);
    app.add_singleton_model(CloudModel::mock);
    let profile_model = app.add_singleton_model(|ctx| {
        AIExecutionProfilesModel::new(&LaunchMode::new_for_unit_test(), ctx)
    });

    let conversation_id = history.update(app, |history_model, ctx| {
        history_model.start_new_conversation(terminal_view_id, false, ctx)
    });

    PermissionsTestState {
        convo_id: conversation_id,
        permissions,
        history,
        terminal_view_id,
        profile_model,
    }
}

#[test]
fn test_can_read_files_empty_paths() {
    App::test((), |mut app| async move {
        let PermissionsTestState {
            convo_id,
            permissions,
            terminal_view_id,
            ..
        } = initialize_permissions_test(&mut app);

        permissions.read(&app, |model, ctx| {
            let result = model.can_read_files_with_conversation(
                &convo_id,
                vec![],
                Some(terminal_view_id),
                ctx,
            );
            assert!(result.is_allowed());
            assert!(matches!(
                result,
                FileReadPermission::Allowed(FileReadPermissionAllowedReason::ExplicitlyAllowlisted)
            ));
        });
    })
}

#[test]
fn test_can_read_files_profile_allowlist_interaction() {
    App::test((), |mut app| async move {
        let PermissionsTestState {
            convo_id,
            permissions,
            profile_model,
            terminal_view_id,
            ..
        } = initialize_permissions_test(&mut app);

        profile_model.update(&mut app, |model, ctx| {
            model.set_read_files(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &ActionPermission::AlwaysAsk,
                ctx,
            );
            model.add_to_directory_allowlist(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &PathBuf::from("/profile/allowed"),
                ctx,
            );
        });

        // Test that files in profile's allowlist are allowed
        permissions.read(&app, |model, ctx| {
            let result = model.can_read_files_with_conversation(
                &convo_id,
                vec![PathBuf::from("/profile/allowed/file.txt")],
                Some(terminal_view_id),
                ctx,
            );
            assert!(result.is_allowed());
            assert!(matches!(
                result,
                FileReadPermission::Allowed(FileReadPermissionAllowedReason::ExplicitlyAllowlisted)
            ));

            // Test that files not in profile's allowlist are denied
            let result = model.can_read_files_with_conversation(
                &convo_id,
                vec![PathBuf::from("/not/allowed/file.txt")],
                Some(terminal_view_id),
                ctx,
            );
            assert!(!result.is_allowed());
            assert!(matches!(
                result,
                FileReadPermission::Denied(FileReadPermissionDeniedReason::AlwaysAskEnabled)
            ));
        });
    })
}

#[test]
fn test_can_write_files() {
    App::test((), |mut app| async move {
        let PermissionsTestState {
            terminal_view_id,
            convo_id,
            permissions,
            profile_model,
            ..
        } = initialize_permissions_test(&mut app);

        // Test AgentDecides setting
        profile_model.update(&mut app, |model, ctx| {
            model.set_apply_code_diffs(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &ActionPermission::AgentDecides,
                ctx,
            );
        });

        permissions.read(&app, |model, ctx| {
            let result = model.can_write_files(&convo_id, &[], Some(terminal_view_id), ctx);
            assert!(!result.is_allowed());
            assert!(
                matches!(
                    result,
                    FileWritePermission::Denied(FileWritePermissionDeniedReason::AgentDecided)
                ),
                "not allowed because AgentDecides right now just means ask"
            );
        });

        // Test AlwaysAllow setting
        profile_model.update(&mut app, |model, ctx| {
            model.set_apply_code_diffs(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &ActionPermission::AlwaysAllow,
                ctx,
            );
        });

        permissions.read(&app, |model, ctx| {
            let result = model.can_write_files(&convo_id, &[], Some(terminal_view_id), ctx);
            assert!(result.is_allowed());
            assert!(matches!(
                result,
                FileWritePermission::Allowed(
                    FileWritePermissionAllowedReason::AutowriteSettingEnabled
                )
            ));
        });

        // Test AlwaysAsk setting
        profile_model.update(&mut app, |model, ctx| {
            model.set_apply_code_diffs(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &ActionPermission::AlwaysAsk,
                ctx,
            );
        });

        permissions.read(&app, |model, ctx| {
            let result = model.can_write_files(&convo_id, &[], Some(terminal_view_id), ctx);
            assert!(!result.is_allowed());
            assert!(matches!(
                result,
                FileWritePermission::Denied(FileWritePermissionDeniedReason::AlwaysAskEnabled)
            ));
        });
    })
}

#[test]
fn test_can_autoexecute_command_denylist_precedence() {
    App::test((), |mut app| async move {
        let PermissionsTestState {
            convo_id,
            permissions,
            profile_model,
            terminal_view_id,
            ..
        } = initialize_permissions_test(&mut app);

        profile_model.update(&mut app, |model, ctx| {
            model.add_to_command_denylist(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &AgentModeCommandExecutionPredicate::new_regex("rm .*").unwrap(),
                ctx,
            );
        });

        permissions.read(&app, |model, ctx| {
            let result = model.can_autoexecute_command(
                &convo_id,
                "rm file.txt",
                EscapeChar::Backslash,
                false,
                None,
                Some(terminal_view_id),
                ctx,
            );
            assert!(!result.is_allowed());
            assert!(matches!(
                result,
                CommandExecutionPermission::Denied(
                    CommandExecutionPermissionDeniedReason::ExplicitlyDenylisted
                )
            ));
        });
    })
}

#[test]
fn test_can_autoexecute_command_allowlist_precedence() {
    App::test((), |mut app| async move {
        let PermissionsTestState {
            convo_id,
            permissions,
            profile_model,
            terminal_view_id,
            ..
        } = initialize_permissions_test(&mut app);

        profile_model.update(&mut app, |model, ctx| {
            model.set_execute_commands(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &ActionPermission::AlwaysAsk,
                ctx,
            );
            model.add_to_command_allowlist(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &AgentModeCommandExecutionPredicate::new_regex("git .*").unwrap(),
                ctx,
            );
        });

        permissions.read(&app, |model, ctx| {
            let result = model.can_autoexecute_command(
                &convo_id,
                "git status",
                EscapeChar::Backslash,
                false,
                None,
                Some(terminal_view_id),
                ctx,
            );
            assert!(result.is_allowed());
            assert!(matches!(
                result,
                CommandExecutionPermission::Allowed(
                    CommandExecutionPermissionAllowedReason::ExplicitlyAllowlisted
                )
            ));
        });
    })
}

#[test]
fn test_can_autoexecute_command_denylist_beats_run_to_completion() {
    App::test((), |mut app| async move {
        let PermissionsTestState {
            convo_id,
            permissions,
            history,
            profile_model,
            terminal_view_id,
            ..
        } = initialize_permissions_test(&mut app);

        // Add a denylist rule that matches the test command.
        profile_model.update(&mut app, |model, ctx| {
            model.add_to_command_denylist(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &AgentModeCommandExecutionPredicate::new_regex("rm .*").unwrap(),
                ctx,
            );
        });

        // Toggle run-to-completion override for this conversation.
        history.update(&mut app, |history, ctx| {
            history.toggle_autoexecute_override(&convo_id, terminal_view_id, ctx);
        });

        // Despite run-to-completion, denylist must take precedence and deny execution.
        permissions.read(&app, |model, ctx| {
            let result = model.can_autoexecute_command(
                &convo_id,
                "rm important.txt",
                EscapeChar::Backslash,
                false,
                None,
                Some(terminal_view_id),
                ctx,
            );
            assert!(!result.is_allowed());
            assert!(matches!(
                result,
                CommandExecutionPermission::Denied(
                    CommandExecutionPermissionDeniedReason::ExplicitlyDenylisted
                )
            ));
        });
    })
}

#[test]
fn test_can_autoexecute_command_run_to_completion_allows_non_denylisted() {
    App::test((), |mut app| async move {
        let PermissionsTestState {
            convo_id,
            permissions,
            history,
            terminal_view_id,
            ..
        } = initialize_permissions_test(&mut app);

        // Enable run-to-completion override for the conversation.
        history.update(&mut app, |history, ctx| {
            history.toggle_autoexecute_override(&convo_id, terminal_view_id, ctx);
        });

        // Since the command is not denylisted, the override should allow execution with RunToCompletion.
        permissions.read(&app, |model, ctx| {
            let result = model.can_autoexecute_command(
                &convo_id,
                "echo hello",
                EscapeChar::Backslash,
                true,        // read-only command
                Some(false), // not risky
                Some(terminal_view_id),
                ctx,
            );
            assert!(result.is_allowed());
            assert!(matches!(
                result,
                CommandExecutionPermission::Allowed(
                    CommandExecutionPermissionAllowedReason::RunToCompletion
                )
            ));
        });
    })
}

#[test]
fn test_can_autoexecute_command_agent_decides_allows_not_risky() {
    App::test((), |mut app| async move {
        let PermissionsTestState {
            convo_id,
            permissions,
            profile_model,
            terminal_view_id,
            ..
        } = initialize_permissions_test(&mut app);

        profile_model.update(&mut app, |model, ctx| {
            model.set_execute_commands(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &ActionPermission::AgentDecides,
                ctx,
            );
        });

        permissions.read(&app, |model, ctx| {
            let result = model.can_autoexecute_command(
                &convo_id,
                "make test",
                EscapeChar::Backslash,
                false,
                Some(false),
                Some(terminal_view_id),
                ctx,
            );
            assert!(result.is_allowed());
            assert!(matches!(
                result,
                CommandExecutionPermission::Allowed(
                    CommandExecutionPermissionAllowedReason::AgentDecided
                )
            ));
        });
    })
}

#[test]
fn test_can_write_to_pty() {
    App::test((), |mut app| async move {
        let PermissionsTestState {
            convo_id,
            permissions,
            profile_model,
            terminal_view_id,
            ..
        } = initialize_permissions_test(&mut app);

        profile_model.update(&mut app, |model, ctx| {
            model.set_write_to_pty(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &WriteToPtyPermission::AlwaysAllow,
                ctx,
            );
        });

        permissions.read(&app, |model, ctx| {
            let result = model.can_write_to_pty(&convo_id, Some(terminal_view_id), ctx);
            assert_eq!(result, WriteToPtyPermission::AlwaysAllow);
        });
    })
}

#[test]
fn test_denylist_matches_multiline_commands() {
    App::test((), |mut app| async move {
        let PermissionsTestState {
            convo_id,
            permissions,
            profile_model,
            terminal_view_id,
            ..
        } = initialize_permissions_test(&mut app);

        // Add denylist rule for rm
        profile_model.update(&mut app, |model, ctx| {
            model.add_to_command_denylist(
                *model.active_profile(Some(terminal_view_id), ctx).id(),
                &AgentModeCommandExecutionPredicate::new_regex("rm .*").unwrap(),
                ctx,
            );
        });

        // Single-line rm command should be denied
        permissions.read(&app, |model, ctx| {
            let result = model.can_autoexecute_command(
                &convo_id,
                "rm file.txt",
                EscapeChar::Backslash,
                false,
                None,
                Some(terminal_view_id),
                ctx,
            );
            assert!(!result.is_allowed());
            assert!(matches!(
                result,
                CommandExecutionPermission::Denied(
                    CommandExecutionPermissionDeniedReason::ExplicitlyDenylisted
                )
            ));
        });

        // Multiline rm command with backslash continuations should also be denied (POSIX)
        permissions.read(&app, |model, ctx| {
            let result = model.can_autoexecute_command(
                &convo_id,
                "rm file1.txt \\\nfile2.txt \\\nfile3.txt",
                EscapeChar::Backslash,
                false,
                None,
                Some(terminal_view_id),
                ctx,
            );
            assert!(
                !result.is_allowed(),
                "multiline rm command should be denied by denylist"
            );
            assert!(matches!(
                result,
                CommandExecutionPermission::Denied(
                    CommandExecutionPermissionDeniedReason::ExplicitlyDenylisted
                )
            ));
        });

        // Multiline rm command with backtick continuations should also be denied (PowerShell)
        permissions.read(&app, |model, ctx| {
            let result = model.can_autoexecute_command(
                &convo_id,
                "rm file1.txt `\nfile2.txt `\nfile3.txt",
                EscapeChar::Backtick,
                false,
                None,
                Some(terminal_view_id),
                ctx,
            );
            assert!(
                !result.is_allowed(),
                "multiline rm command with backtick continuations should be denied by denylist"
            );
            assert!(matches!(
                result,
                CommandExecutionPermission::Denied(
                    CommandExecutionPermissionDeniedReason::ExplicitlyDenylisted
                )
            ));
        });
    })
}
