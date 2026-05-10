# ACP Native Agent UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Do not dispatch subagents for this work. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the remaining Warp-agent-shaped display path with an ACP-native conversation and tool-call UI that uses Warp's design language while matching Zed's ACP event semantics.

**Architecture:** Keep the existing Warp AgentView shell, input area, scroll behavior, typography, and shared inline-action primitives. Add an ACP-native message/tool-call data layer and renderer so ACP events are displayed as ACP entries instead of being converted into old `AIAgentActionType` executor actions. This avoids re-executing ACP tools while making the UI removable independently from the old Warp Agent implementation later.

**Tech Stack:** Rust, Warp UI/GPUI-style components, `agent-client-protocol`, existing `AIAgentOutputMessage` history model, existing `RenderableAction` and inline action components.

---

## File Structure

Create:
- `app/src/ai/acp/thread.rs`: ACP-native display state types and pure merge helpers for `ToolCall`, `ToolCallUpdate`, plan, commands, config, and session info.
- `app/src/ai/acp/permission.rs`: ACP permission request/option data now; add a responder broker here only when interactive permission selection is wired.

Modify:
- `app/src/ai/acp/mod.rs`: Export new ACP modules.
- `app/src/ai/acp/events.rs`: Replace text-only events with ACP-native event variants.
- `app/src/ai/acp/mapping.rs`: Map every ACP `SessionUpdate` variant we support into ACP-native events.
- `app/src/ai/acp/model.rs`: Feed ACP events into history, keep permission requests visible, and advertise only implemented capabilities.
- `app/src/ai/acp/tests.rs`: Add reducer and mapping tests for every supported ACP update.
- `app/src/ai/agent/mod.rs`: Add ACP-native output message variants.
- `app/src/ai/agent/conversation.rs`: Add append/upsert/update methods for ACP messages and avoid merging assistant text across tool-call boundaries.
- `app/src/ai/blocklist/history_model.rs`: Add wrappers that route ACP updates into the active conversation.
- `app/src/ai/blocklist/block/view_impl/output.rs`: Render ACP-native output message variants via existing Warp components. Keep the renderer here while it is small; split into `inline_action/acp_tool_call.rs` or `inline_action/acp_permission.rs` only when content/diff/permission interaction complexity justifies it.
- `app/src/ai/blocklist/agent_view/agent_input_footer/mod.rs`: Show ACP config/mode controls only when the current conversation is ACP.
- `app/src/terminal/input/slash_commands/data_source/mod.rs`: Surface ACP `AvailableCommandsUpdate` commands in the AgentView command path.

Do not copy the full Warp Agent UI tree. Copying the whole tree would bring old cloud-agent action execution, shared-session assumptions, and orchestration controls into ACP. The new ACP renderer should reuse shared primitives and small existing components, not old execution semantics.

---

### Task 1: ACP Display State Types

**Files:**
- Create: `app/src/ai/acp/thread.rs`
- Modify: `app/src/ai/acp/mod.rs`
- Test: `app/src/ai/acp/tests.rs`

- [x] **Step 1: Write failing reducer tests**

Add tests that prove a `ToolCall` creates state, `ToolCallUpdate` updates the same id, raw output is preserved, and unknown updates do not panic.

```rust
#[test]
fn test_acp_tool_call_update_merges_existing_call() {
    use agent_client_protocol::schema::{
        ToolCall, ToolCallStatus, ToolCallUpdate, ToolCallUpdateFields, ToolKind,
    };

    let mut call = AcpToolCall::from_acp(
        ToolCall::new("read-1", "Read SKILL.md")
            .kind(ToolKind::Read)
            .status(ToolCallStatus::InProgress),
    );

    call.apply_update(ToolCallUpdate::new(
        "read-1",
        ToolCallUpdateFields::new().status(ToolCallStatus::Completed),
    ));

    assert_eq!(call.id.as_str(), "read-1");
    assert_eq!(call.title, "Read SKILL.md");
    assert_eq!(call.kind, ToolKind::Read);
    assert_eq!(call.status, ToolCallStatus::Completed);
}
```

- [x] **Step 2: Run the failing test**

Run: `cargo test -p warp --lib ai::acp::tests::test_acp_tool_call_update_merges_existing_call --locked`

Expected: fail because `AcpToolCall` does not exist yet.

- [x] **Step 3: Implement minimal ACP state types**

Create `app/src/ai/acp/thread.rs` with:

```rust
use agent_client_protocol::schema::{
    AvailableCommand, ConfigOptionUpdate, CurrentModeUpdate, Plan, SessionInfoUpdate, ToolCall,
    ToolCallContent, ToolCallLocation, ToolCallStatus, ToolCallUpdate, ToolKind,
};

#[derive(Clone, Debug, PartialEq)]
pub struct AcpToolCall {
    pub id: String,
    pub title: String,
    pub kind: ToolKind,
    pub status: ToolCallStatus,
    pub content: Vec<ToolCallContent>,
    pub locations: Vec<ToolCallLocation>,
    pub raw_input: Option<serde_json::Value>,
    pub raw_output: Option<serde_json::Value>,
    pub meta: Option<serde_json::Map<String, serde_json::Value>>,
}

impl AcpToolCall {
    pub fn from_acp(call: ToolCall) -> Self {
        Self {
            id: call.tool_call_id.0.to_string(),
            title: call.title,
            kind: call.kind,
            status: call.status,
            content: call.content,
            locations: call.locations,
            raw_input: call.raw_input,
            raw_output: call.raw_output,
            meta: call.meta,
        }
    }

    pub fn apply_update(&mut self, update: ToolCallUpdate) {
        if let Some(title) = update.fields.title {
            self.title = title;
        }
        if let Some(kind) = update.fields.kind {
            self.kind = kind;
        }
        if let Some(status) = update.fields.status {
            self.status = status;
        }
        if let Some(content) = update.fields.content {
            self.content = content;
        }
        if let Some(locations) = update.fields.locations {
            self.locations = locations;
        }
        if let Some(raw_input) = update.fields.raw_input {
            self.raw_input = Some(raw_input);
        }
        if let Some(raw_output) = update.fields.raw_output {
            self.raw_output = Some(raw_output);
        }
        if update.meta.is_some() {
            self.meta = update.meta;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AcpPlan {
    pub plan: Plan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpCommands {
    pub commands: Vec<AvailableCommand>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpSessionConfig {
    pub current_mode: Option<CurrentModeUpdate>,
    pub config: Option<ConfigOptionUpdate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpSessionInfo {
    pub info: SessionInfoUpdate,
}
```

Export from `app/src/ai/acp/mod.rs`:

```rust
mod thread;
pub use thread::*;
```

- [x] **Step 4: Run tests**

Run: `cargo test -p warp --lib ai::acp --locked`

Expected: pass.

- [x] **Step 5: Run compile check**

Run: `cargo check -p warp --lib --locked`

Expected: pass or only unrelated existing warnings.

---

### Task 2: ACP Event Mapping Coverage

**Files:**
- Modify: `app/src/ai/acp/events.rs`
- Modify: `app/src/ai/acp/mapping.rs`
- Test: `app/src/ai/acp/tests.rs`

- [x] **Step 1: Write failing mapping tests**

Add one test per ACP update family:

```rust
#[test]
fn test_map_tool_call_update_to_acp_event() {
    use agent_client_protocol::schema::{SessionUpdate, ToolCallUpdate, ToolCallUpdateFields};

    let update = SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
        "tool-1",
        ToolCallUpdateFields::new(),
    ));

    assert!(matches!(
        map_session_update(update),
        Some(AcpEvent::ToolCallUpdated { .. })
    ));
}
```

- [x] **Step 2: Run the failing mapping tests**

Run: `cargo test -p warp --lib ai::acp::tests::test_map_tool_call_update_to_acp_event --locked`

Expected: fail because `ToolCallUpdated` does not exist or is not mapped.

- [x] **Step 3: Expand `AcpEvent`**

Replace the display event enum with variants for the ACP protocol surface:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum AcpEvent {
    AdapterMissing { command: String, install_command: String },
    SessionStarted,
    UserTextDelta { text: String },
    AssistantTextDelta { text: String },
    AssistantThoughtDelta { text: String },
    ToolCallStarted { tool_call: AcpToolCall },
    ToolCallUpdated { update: agent_client_protocol::schema::ToolCallUpdate },
    PlanUpdated { plan: agent_client_protocol::schema::Plan },
    AvailableCommandsUpdated { commands: Vec<agent_client_protocol::schema::AvailableCommand> },
    CurrentModeUpdated { update: agent_client_protocol::schema::CurrentModeUpdate },
    ConfigOptionsUpdated { update: agent_client_protocol::schema::ConfigOptionUpdate },
    SessionInfoUpdated { update: agent_client_protocol::schema::SessionInfoUpdate },
    PermissionRequested { request: AcpPermissionRequest },
    Completed,
    Failed { message: String },
}
```

Define `AcpPermissionRequest` in `app/src/ai/acp/permission.rs` during Task 4. Until then, add the type in `events.rs` with the request id, tool call update, and options.

- [x] **Step 4: Map all stable `SessionUpdate` variants**

Update `map_session_update`:

```rust
match update {
    SessionUpdate::UserMessageChunk(chunk) => text_chunk(chunk).map(AcpEvent::UserTextDelta),
    SessionUpdate::AgentMessageChunk(chunk) => text_chunk(chunk).map(AcpEvent::AssistantTextDelta),
    SessionUpdate::AgentThoughtChunk(chunk) => text_chunk(chunk).map(AcpEvent::AssistantThoughtDelta),
    SessionUpdate::ToolCall(tool_call) => Some(AcpEvent::ToolCallStarted {
        tool_call: AcpToolCall::from_acp(tool_call),
    }),
    SessionUpdate::ToolCallUpdate(update) => Some(AcpEvent::ToolCallUpdated { update }),
    SessionUpdate::Plan(plan) => Some(AcpEvent::PlanUpdated { plan }),
    SessionUpdate::AvailableCommandsUpdate(update) => Some(AcpEvent::AvailableCommandsUpdated {
        commands: update.available_commands,
    }),
    SessionUpdate::CurrentModeUpdate(update) => Some(AcpEvent::CurrentModeUpdated { update }),
    SessionUpdate::ConfigOptionUpdate(update) => Some(AcpEvent::ConfigOptionsUpdated { update }),
    SessionUpdate::SessionInfoUpdate(update) => Some(AcpEvent::SessionInfoUpdated { update }),
    _ => None,
}
```

- [x] **Step 5: Run mapping tests and compile**

Run:
- `cargo test -p warp --lib ai::acp --locked`
- `cargo check -p warp --lib --locked`

Expected: pass.

---

### Task 3: ACP-Native Output Messages

**Files:**
- Modify: `app/src/ai/agent/mod.rs`
- Modify: `app/src/ai/agent/conversation.rs`
- Modify: `app/src/ai/blocklist/history_model.rs`
- Test: existing conversation/history tests or new tests near ACP helpers

- [x] **Step 1: Add failing conversation tests**

Test that text before and after a tool call becomes two separate assistant messages:

```rust
#[test]
fn test_acp_text_does_not_merge_across_tool_call() {
    let stream_id = ResponseStreamId::new("stream-1".to_string());
    let tool_call = AcpToolCall::from_acp(
        agent_client_protocol::schema::ToolCall::new("tool-1", "Read SKILL.md"),
    );

    conversation.append_local_text_delta_to_response_stream(&stream_id, terminal_id, "before", model, name, ctx).unwrap();
    conversation.upsert_acp_tool_call_to_response_stream(&stream_id, terminal_id, tool_call, model, name, ctx).unwrap();
    conversation.append_local_text_delta_to_response_stream(&stream_id, terminal_id, "after", model, name, ctx).unwrap();

    let output = conversation.streaming_output_for_response(&stream_id).unwrap();
    assert_eq!(output.messages.len(), 3);
}
```

- [x] **Step 2: Add output message variants**

In `AIAgentOutputMessageType`, add:

```rust
AcpToolCall(AcpToolCall),
AcpPlan(AcpPlan),
AcpPermission(AcpPermissionRequest),
```

Add constructors on `AIAgentOutputMessage`:

```rust
pub fn acp_tool_call(id: MessageId, tool_call: AcpToolCall) -> Self
pub fn acp_plan(id: MessageId, plan: AcpPlan) -> Self
pub fn acp_permission(id: MessageId, request: AcpPermissionRequest) -> Self
```

- [x] **Step 3: Add conversation append/upsert/update helpers**

Implement:

```rust
pub fn upsert_acp_tool_call_to_response_stream(...)
pub fn update_acp_tool_call_to_response_stream(...)
pub fn set_acp_plan_for_response_stream(...)
pub fn upsert_acp_permission_to_response_stream(...)
```

Use message ids:
- `local-acp-{stream}-tool-{tool_call_id}`
- `local-acp-{stream}-plan`
- `local-acp-{stream}-permission-{request_id}`

Change text appending so it appends only to the last output message when that last message is `Text`; otherwise push a new text message id with a monotonically increasing suffix.

- [x] **Step 4: Add history model wrappers**

Add matching methods in `BlocklistAIHistoryModel` that delegate to the active conversation.

- [x] **Step 5: Run focused tests and compile**

Run:
- `cargo test -p warp --lib acp --locked`
- `cargo test -p warp --lib ai::acp --locked`
- `cargo check -p warp --lib --locked`

Expected: pass.

---

### Task 4: Generic ACP Tool-Call Rendering

**Files:**
- Modify: `app/src/ai/blocklist/block/view_impl/output.rs`
- Test: `app/src/ai/blocklist/block/view_impl/output_tests.rs`

- [x] **Step 1: Add failing render tests**

Add tests that a completed read call renders `Read SKILL.md`, an edit call renders diff content, and an execute call renders terminal/code output when content is present.

- [x] **Step 2: Implement `render_acp_tool_call`**

The renderer accepts `&AcpToolCall` and returns `Box<dyn Element>`. Use existing `RenderableAction::new_with_formatted_text` for text rows. Map status:
- `Pending`: muted row
- `InProgress`: normal row with in-progress footer
- `Completed`: completed icon/state
- `Failed`: error icon/state

Map kind:
- `Read`: read/search icon and path/title
- `Edit/Delete/Move`: diff/file edit styling
- `Search/Fetch`: retrieval styling
- `Execute`: command/output styling
- `Think`: thinking/action row
- `Other/SwitchMode`: generic row

For ACP read calls that include a standard `ToolCallLocation` pointing exactly at a recognized skill `SKILL.md`, reuse the existing Warp ReadSkill row renderer with an ACP status icon. Do not infer skill reads from the title, model wording, adapter-specific metadata, or missing fields.

- [x] **Step 3: Render ACP content**

Support `ToolCallContent`:
- [x] `Content(Text)`: existing formatted text
- [x] `Content(ResourceLink/Resource)`: existing link-style text with real link/open behavior
- [x] `Content(Image)`: dedicated image rendering using existing image output components
- [x] `Diff`: existing `CodeDiffView::new_passive` in view-only embedded mode
- [x] `Terminal`: dedicated terminal rendering in Task 7; do not render terminal ids as plain text

- [x] **Step 4: Wire output branch**

In `output.rs`, add branches for:

```rust
AIAgentOutputMessageType::AcpToolCall(tool_call) => { ... }
AIAgentOutputMessageType::AcpPlan(plan) => { ... }
AIAgentOutputMessageType::AcpPermission(request) => { ... }
```

Do not add these to `output.actions()`, and do not queue them in `BlocklistAIActionModel`.

- [x] **Step 5: Run UI compile checks**

Run:
- `cargo test -p warp --lib ai::blocklist::block::view_impl::output_tests --locked`
- `cargo check -p warp --lib --locked`
- `cargo fmt --all --check`

Expected: pass.

---

### Task 5: ACP Permission Flow

**Files:**
- Modify: `app/src/ai/acp/permission.rs`
- Create only if needed: `app/src/ai/blocklist/inline_action/acp_permission.rs`
- Modify: `app/src/ai/acp/model.rs`
- Modify: `app/src/ai/acp/events.rs`
- Modify: `app/src/ai/blocklist/block/view_impl/output.rs`
- Test: `app/src/ai/acp/tests.rs`

- [x] **Step 1: Write failing permission tests**

Test that permission requests do not auto-allow and that selecting an option returns the selected ACP option id.

- [x] **Step 2: Implement permission request data**

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct AcpPermissionRequest {
    pub request_id: String,
    pub tool_call_id: String,
    pub tool_call_update: agent_client_protocol::schema::ToolCallUpdate,
    pub options: Vec<AcpPermissionOption>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpPermissionOption {
    pub option_id: String,
    pub name: String,
    pub kind: agent_client_protocol::schema::PermissionOptionKind,
}
```

- [x] **Step 3: Implement broker**

`AcpPermissionBroker` stores pending responders by request id. `on_receive_request` registers the responder, emits `PermissionRequested`, waits for user selection, then returns `RequestPermissionResponse`.

- [x] **Step 4: Render permission UI**

Use existing action button themes. Buttons map directly to ACP options. Do not show old Warp auto-approve controls in this ACP path.

- [x] **Step 5: Run tests**

Run:
- `cargo test -p warp --lib ai::acp --locked`
- `cargo check -p warp --lib --locked`

Expected: pass.

---

### Task 6: Plan, Commands, Config, and Session Info

**Files:**
- Modify: `app/src/ai/acp/model.rs`
- Modify: `app/src/ai/agent/conversation.rs`
- Modify: `app/src/ai/blocklist/history_model.rs`
- Modify: `app/src/ai/blocklist/agent_view/agent_input_footer/mod.rs`
- Modify: `app/src/terminal/input/slash_commands/data_source/mod.rs`
- Test: `app/src/ai/acp/tests.rs`, `app/src/terminal/input/slash_command_model_tests.rs`

- [x] **Step 1: Plan rendering**

Store ACP `Plan` as `AcpPlan` and render it using existing todo/plan visual language. Do not append plan as plain assistant text.

- [x] **Step 2: Available commands**

Store `AvailableCommandsUpdate` per active ACP session. In AgentView command mode, expose those commands as ACP commands and pass command input to the ACP prompt path.

- [x] **Step 3: Mode/config updates**

When `CurrentModeUpdate` or `ConfigOptionUpdate` arrives, update the active ACP config state. The footer/settings UI renders the active session values and calls ACP `set_session_mode` or `set_session_config_option` instead of writing fixed hardcoded values.

- [x] **Step 4: Session info**

When `SessionInfoUpdate.title` is present, update the conversation title. Preserve existing title if the update field is undefined.

- [x] **Step 5: Run tests**

Run:
- `cargo test -p warp --lib ai::acp --locked`
- `cargo test -p warp --lib terminal::input::slash_command_model_tests --locked`
- `cargo check -p warp --lib --locked`

Expected: pass.

---

### Task 7: Terminal and File-System ACP Capabilities

**Files:**
- Modify: `app/src/ai/acp/model.rs`
- Modify: `app/src/ai/acp/thread.rs`
- Modify: `app/src/ai/blocklist/inline_action/acp_tool_call.rs`
- Test: `app/src/ai/acp/tests.rs`

- [x] **Step 1: Keep capabilities conservative first**

Do not advertise `client_capabilities.terminal = true` until all terminal request handlers exist. Unsupported terminal capability must remain unadvertised instead of being represented as plain-text output.

- [x] **Step 2: Support codex-acp terminal meta**

Handle `terminal_info`, `terminal_output`, and `terminal_exit` metadata carried on `ToolCall` and `ToolCallUpdate` as structured terminal trace state. Do not append this data to assistant text or generic tool-call text; render it only through the ACP terminal UI path once that UI is wired.

- [x] **Step 3: Add real terminal request handlers**

Only after Step 2 works, implement ACP `terminal/create`, `terminal/output`, `terminal/release`, `terminal/wait_for_exit`, and `terminal/kill` request handlers and then advertise `client_capabilities.terminal = true`.

- [x] **Step 4: Add fs request handlers if needed by selected adapters**

Implement `fs/read_text_file` and `fs/write_text_file` only if Codex/Claude ACP uses them in practice. Gate capabilities so unsupported handlers are not advertised.

- [x] **Step 5: Run tests and compile**

Run:
- `cargo test -p warp --lib ai::acp --locked`
- `cargo check -p warp --lib --locked`

Expected: pass.

---

### Task 8: Remove Remaining Warp-Agent-Only Controls From ACP Path

**Files:**
- Modify: `app/src/ai/blocklist/agent_view/agent_input_footer/mod.rs`
- Modify: `app/src/ai/blocklist/agent_view/agent_message_bar.rs`
- Modify: `app/src/ai/blocklist/agent_view/zero_state_block.rs`
- Modify: `app/src/terminal/view/agent_view.rs`
- Test: existing zero-state/footer tests

- [x] **Step 1: Hide old controls only for ACP conversations**

Remove or hide old cloud-agent controls from ACP conversations:
- remote-control
- Warp cloud agent references
- Warp credit usage hints
- old auto-approve controls that do not map to ACP permission options
- old orchestration child-agent controls unless they are emitted as ACP tool calls

- [x] **Step 2: Keep shared shell controls**

Keep controls that are genuinely local and UI-level:
- send/cancel
- model/config selectors backed by ACP config options
- prompt editor
- conversation continuation

- [x] **Step 3: Run UI tests and compile**

Run:
- `cargo test -p warp --lib ai::blocklist::agent_view --locked`
- `cargo check -p warp --lib --locked`

Expected: pass.

---

### Task 9: End-to-End Verification

**Files:**
- No new files unless a focused integration test already exists for AgentView ACP.

- [x] **Step 1: Unit test pass**

Run:
- `cargo test -p warp --lib ai::acp --locked`
- `cargo test -p warp --lib acp --locked`
- `cargo test -p warp --lib ai::blocklist::block::view_impl::output_tests --locked`

Expected: pass.

- [x] **Step 2: Compile**

Run: `cargo check -p warp --lib --locked`

Expected: pass or only unrelated existing warnings.

- [x] **Step 3: Format**

Run: `cargo fmt --all --check`

Expected: pass.

- [x] **Step 4: Whitespace diff check**

Run: `git diff --check`

Expected: no output.

- [ ] **Step 5: Manual behavior check**

Launch Warp locally, open AgentView, select Codex ACP, and verify:
- `/agent 你好` creates a conversation entry
- tool calls such as `Read SKILL.md` render as separate rows
- assistant text before and after tool calls is not merged incorrectly
- permission prompts wait for user input
- plan updates render as plan UI
- config/mode selectors reflect adapter-provided options
- no Warp login/credits/remote-control UI appears in ACP conversations

---

## Self-Review

Spec coverage:
- ACP message chunks: Task 2 and Task 3.
- ACP tool calls and updates: Task 1 through Task 4.
- ACP permissions: Task 5.
- ACP plan, commands, config, and title: Task 6.
- Terminal and fs capabilities: Task 7.
- Warp Agent UI cleanup for ACP path: Task 8.
- Frequent tests and compile checks: every task plus Task 9.

Design decision:
- We do not copy the entire Warp Agent UI. We create an ACP renderer that reuses Warp's existing visual primitives. This gives the future deletion path the user wants because ACP data and rendering are isolated from the old cloud-agent executor.
