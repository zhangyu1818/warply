use std::collections::HashMap;

use warpui::{App, ModelHandle};

use super::super::diff_state_tracker::RemoteDiffStateManager;
use super::super::protocol::RequestId;
use super::super::server_buffer_tracker::ServerBufferTracker;
use super::{PendingFileOps, ServerModel};

fn test_model_with_diff_states(diff_states: ModelHandle<RemoteDiffStateManager>) -> ServerModel {
    ServerModel {
        connection_senders: HashMap::new(),
        snapshot_sent_roots_by_connection: HashMap::new(),
        grace_timer_cancel: None,
        in_progress: HashMap::new(),
        host_id: "test-host-id".to_string(),
        executors: HashMap::new(),
        pending_file_ops: PendingFileOps::new(),
        buffers: ServerBufferTracker::new(),
        diff_states,
    }
}

fn test_model_handle(app: &mut App) -> ModelHandle<ServerModel> {
    app.add_model(|ctx| {
        let diff_states = ctx.add_model(|_| RemoteDiffStateManager::new());
        test_model_with_diff_states(diff_states)
    })
}

#[cfg(unix)]
#[path = "server_model_unix_tests.rs"]
mod unix;
