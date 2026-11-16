use warpui::{AppContext, SingletonEntity};

use super::HookRuntime;
use super::envelope::{EnvelopeSite, envelope_for_session_start};
use crate::ai::agent::conversation::AIConversationId;

/// Fires `session.start` for a conversation this client just created and will drive. The
/// envelope has no working directory because conversation creation happens before any session
/// is attached to it.
pub(crate) fn notify_session_start(conversation_id: AIConversationId, ctx: &mut AppContext) {
    if !HookRuntime::is_active(ctx) {
        return;
    }
    // The fork has no autonomous background agents; every conversation is interactive.
    let is_background_agent = false;
    let site = EnvelopeSite::for_new_conversation(conversation_id);
    let envelope = envelope_for_session_start(is_background_agent, &site, ctx);
    HookRuntime::as_ref(ctx).dispatch_detached(envelope, ctx);
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
