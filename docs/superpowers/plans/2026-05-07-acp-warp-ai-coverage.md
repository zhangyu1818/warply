# ACP 覆盖 Warp AI 功能实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Do not dispatch subagents for this work. Steps use checkbox (`- [ ]`) syntax for tracking.

**目标：** 重新审计并修正当前 ACP Agent 接入，让它精确替换旧 Warp Agent 的后端 AI 流程，同时保留与后端无关的 Warp 通用 UI 能力。

**架构：** AgentView 未来就是 ACP View。输入框、会话导航、右侧 code review/diff 面板、context chips、slash commands 菜单继续作为 Warp GUI 外壳存在；会话请求、协议事件、工具调用、权限请求、计划、终端/文件能力和输出渲染只走 ACP。旧 Warp 云端 Agent、共享/登录/remote-control/Drive/Teams 语义直接移除。所有 ACP UI 必须按协议字段精确渲染，不做标题推断，不补造协议外数据。

**Tech Stack:** Rust、Warp UI 组件、`agent-client-protocol`、现有 `AgentView`、`AIConversation`、`BlocklistAIHistoryModel`、本地 OpenAI-compatible suggestions client。

---

## 扫描结论

### 可由 ACP 覆盖的旧 Warp AI 后端能力

- 自然语言输入入口：旧流程在 `app/src/terminal/input.rs` 做 NLD，再通过 `AgentViewEntryOrigin::Input { was_prompt_autodetected }` 进入 `AgentView`，最终走 `BlocklistAIController::send_user_query_in_conversation`。当前 ACP 已在 `app/src/ai/blocklist/controller.rs` 的请求提交处接管，这条路径应继续复用。
- `/agent` 入口：旧流程通过 slash command 进入相同 `AgentView`，当前也应继续进入 ACP 请求路径。
- assistant 正文和 thought：ACP `AgentMessageChunk` / `AgentThoughtChunk` 已映射到 `Text` / `Reasoning` 消息，应该继续使用旧 Warp output renderer，而不是混在同一文本块里。
- tool call：ACP `ToolCall` / `ToolCallUpdate` 已进入 `AcpToolCall`，可由 Warp 现有 `RenderableAction`、diff view、read skill 行组件承载。
- plan：ACP `Plan` 由 AgentView 输出区渲染为 ACP plan card，使用 Warp inline action 的视觉语言，不再拼成通用文本行。
- permission：ACP `RequestPermissionRequest` 渲染为 ACP permission card，并通过协议 option 精确回写用户选择。
- available commands：ACP `AvailableCommandsUpdate` 可以进入 AgentView 的 `/` slash commands 菜单。当前已有数据源接入，应补足显示/执行验证。
- terminal capability：ACP client capability 声明了 terminal，当前通过 `AcpTerminalManager` 执行隐藏子进程并把 trace 显示在 tool call footer。
- read/write text file capability：ACP client capability 声明了 `read_text_file` / `write_text_file`，当前直接访问本地文件系统。

### 不应由 ACP 覆盖、但应保留的 Warp 通用 UI 能力

- `⇧⌘+ for code review`：这是 `WorkspaceAction::ToggleRightPanel` / `TerminalAction::ToggleCodeReviewPane`，打开右侧 Git diff/code review 面板，不属于旧 Warp Agent 后端，也不属于 ACP 后端。ACP AgentView 下应继续可用，不能因为是 ACP conversation 就隐藏。
- `⌘Y open conversation` / `/conversations`：这是 Warp 会话导航 UI，依赖 `ConversationNavigationData` 和 `BlocklistAIHistoryModel`。ACP conversation 仍然是 `AIConversationId`，所以必须保留；如果当前导航或持久化没有接通 ACP conversation，就修导航和持久化，不通过隐藏入口绕过问题。
- `? for help`：这是 AgentView 快捷键帮助 UI。应保留，但文案要 ACP-aware，删除或替换旧 Warp 云端/共享/auto-accept 等不适用条目。
- context chips、`@` context menu、文件 attach：这是 Warp 输入上下文能力。ACP prompt 现在只拼 user query，后续要明确哪些上下文能转成 ACP `ContentBlock` 或文本前缀；不能静默丢上下文。
- Next Command / Prompt Suggestions：这是独立的 OpenAI-compatible suggestions 功能，和 ACP Agent 后端无关。当前本地 provider 已接入，应继续作为 “Terminal Suggestions” 保留。

### 不能由当前 ACP 协议或当前实现直接覆盖的旧 Warp Agent 能力

- Warp 云端 Agent、ambient/cloud agent、handoff to cloud、shared session、remote-control、Warp plugin notifications、登录/团队/Drive/Teams：这些是 Warp 平台能力，不应迁入单机 ACP 路径。
- 旧 Warp action executor 的自动审批/auto-accept 语义：ACP 有自己的 permission request/option 机制，不能把旧 `autoexecute_any_action` 当成 ACP permission 自动批准。
- 旧 Warp MAA 的 server token、cloud metadata、conversation sharing/forking：ACP 会话不应依赖 server token。历史导航可以保留本地 conversation，但云端 fork/share 不应保留。
- 旧 Warp 的 codebase indexing：如果只做本地 ACP GUI，不应继续展示为必需设置；ACP 后端自己决定是否索引或读取文件。

### 当前实现里需要纠偏的点

- `app/src/ai/blocklist/agent_view/agent_message_bar.rs` 里已经出现了 `agent_message_bar_shortcut_visibility` 测试辅助，测试假设 ACP conversation 要隐藏 conversation menu 和 code review。这个方向不准确：AgentView 只按 ACP-only 路径设计，code review 和 conversation menu 都是通用 UI，不能按后端隐藏。
- `AcpRunTarget` 和部分 message id 仍带有旧本地命名。用户已经明确不希望概念上叫本地 ACP，应统一成 ACP 命名。
- ACP tool call content 里存在 meta 解析 terminal trace 的代码。terminal trace 应只来自 ACP client terminal capability：`CreateTerminalRequest`、`TerminalOutputRequest`、`WaitForTerminalExitRequest`、`KillTerminalRequest`、`ReleaseTerminalRequest` 进入 `AcpTerminalManager`，再通过 `ToolCallContent::Terminal` 精确关联到对应 tool call。

---

## 文件结构

修改：

- `app/src/ai/acp/model.rs`：修正命名；保留并扩展事件进入 history 的精准映射；让 AgentView 请求路径直接使用 ACP session state。
- `app/src/ai/acp/thread.rs`：保持 ACP 协议字段类型和纯 merge helper；删除非协议 meta 解析 terminal trace 的逻辑。
- `app/src/ai/acp/tests.rs`：补 ACP session state、event mapping、config/current mode、terminal trace、permission 的覆盖测试。
- `app/src/ai/blocklist/agent_view/agent_message_bar.rs`：把 message bar 改成 ACP-only footer；保留 code review 和 conversation menu 通用入口；删除旧 cloud/auto-accept 条件。
- `app/src/ai/blocklist/agent_view/shortcuts/mod.rs`：给快捷键帮助添加 ACP-aware context，保留通用项，移除旧云端/共享/auto-accept 文案。
- `app/src/terminal/input/agent.rs`：向 shortcuts view 传入 ACP-aware context。
- `app/src/terminal/input/conversations/data_source.rs`：补 ACP conversation 导航测试需要的最小行为，确认当前 conversation 过滤、历史 conversation 展示不破坏。
- `app/src/terminal/input/slash_commands/data_source/mod.rs`：继续让 ACP available commands 进入 slash commands，补空命令/命令更新测试。
- `app/src/terminal/input/slash_commands/mod.rs`：确认选择 ACP command 后提交行为与 ACP adapter 预期一致。
- `app/src/ai/blocklist/block/view_impl/output.rs`：继续按 ACP 协议字段渲染 tool call / plan / permission；只复用旧 Warp UI 已有组件，不做标题推断。
- `app/src/ai/agent/conversation.rs`：验证 ACP-native output 是否能被本地 conversation persistence 正确恢复；必要时添加 ACP output 持久化结构。
- `app/src/ai/blocklist/history_model.rs`：补 ACP conversation metadata/title/status/persistence 相关测试。
- `app/src/settings/ai.rs`、`app/src/settings_view/ai_page.rs`：确认 settings 文案和命名统一为 ACP；Terminal Suggestions 和 ACP Agent 分区保持独立。
- `docs/superpowers/plans/2026-05-07-acp-native-agent-ui.md`：同步标记被本计划替代的剩余 toolbar/coverage 工作。

不创建新的 UI 树。只有当旧 Warp 组件无法表达 ACP 协议字段时，才在对应现有 UI 模块旁边新增 ACP 小组件。

---

### Task 1: message bar 改为 ACP-only 能力边界

**文件：**
- 修改：`app/src/ai/blocklist/agent_view/agent_message_bar.rs`
- 测试：`app/src/ai/blocklist/agent_view/agent_message_bar.rs`

- [x] **Step 1: 写失败测试，证明 ACP footer 保留通用入口**

把现有 `acp_conversations_hide_legacy_shortcuts` 改成：

```rust
#[test]
fn acp_footer_keeps_generic_shortcuts() {
    let visibility = agent_message_bar_shortcut_visibility(false, true);

    assert!(visibility.show_conversation_menu);
    assert!(visibility.show_code_review);
}
```

运行：

```bash
cargo test -p warp --lib ai::blocklist::agent_view::agent_message_bar::tests::acp_footer_keeps_generic_shortcuts --locked
```

预期：当前失败，因为 helper 仍带有 ACP 分支或旧 cloud 条件。

- [x] **Step 2: 最小实现，删除 ACP 分支**

把 helper 改成：

```rust
fn agent_message_bar_shortcut_visibility(
    has_conversation_been_updated_since_agent_view_entry: bool,
    show_code_review_button: bool,
) -> AgentMessageBarShortcutVisibility {
    AgentMessageBarShortcutVisibility {
        show_conversation_menu: !has_conversation_been_updated_since_agent_view_entry,
        show_code_review: show_code_review_button,
    }
}
```

如果生产代码里不需要 helper，就删除 helper 并直接改生产判断；测试改为覆盖渲染结果。

- [x] **Step 3: 运行测试**

```bash
cargo test -p warp --lib ai::blocklist::agent_view::agent_message_bar::tests --locked
```

预期：通过。

---

### Task 2: 接通 ACP conversation 导航

**文件：**
- 修改：`app/src/ai/acp/model.rs`
- 修改：`app/src/terminal/input/conversations/data_source.rs`
- 测试：`app/src/terminal/input/conversations/data_source.rs` 或相邻现有测试模块

- [x] **Step 1: 写导航测试，确认 ACP conversation 仍出现在历史导航中**

测试目标：

```rust
#[test]
fn conversation_menu_includes_completed_acp_conversations() {
    // 创建一个 AIConversation，模拟 ACP 请求完成并写入 history metadata。
    // 调用 ConversationNavigationData::all_conversations(app)。
    // 断言该 conversation 出现在结果里，当前 active conversation 仍被过滤。
}
```

如果现有 test harness 无法轻量创建完整 workspace，就先写 `BlocklistAIHistoryModel` 层测试，验证 `get_local_conversations_metadata` 包含 ACP 完成会话；再补 conversation menu 集成测试。

- [x] **Step 2: 修复导航数据源**

message bar 的 conversation menu 显示规则固定为：

```rust
show_conversation_menu: !has_conversation_been_updated_since_agent_view_entry
```

如果测试失败，修 `ConversationNavigationData::all_conversations`、`BlocklistAIHistoryModel` 或 ACP conversation 写入路径，让完成后的 ACP conversation 进入同一套本地导航索引。

- [x] **Step 3: 运行验证**

```bash
cargo test -p warp --lib terminal::input::conversations --locked
cargo test -p warp --lib ai::blocklist::history_model --locked
```

预期：相关测试通过。

---

### Task 3: 验证 ACP output 本地持久化

**文件：**
- 修改：`app/src/ai/agent/conversation.rs`
- 修改：`app/src/ai/blocklist/history_model.rs`
- 修改：`app/src/ai/blocklist/history_model_test.rs`
- 可能修改：`app/src/ai/blocklist/persistence.rs`

- [x] **Step 1: 写失败测试，覆盖 ACP output 完成后的本地恢复**

新增测试逻辑：

```rust
#[test]
fn acp_output_messages_are_available_after_conversation_persistence_roundtrip() {
    // 创建 conversation。
    // 调用 update_conversation_for_new_request_input。
    // 追加 assistant text、ACP tool call、ACP plan、ACP permission。
    // mark_response_stream_completed_successfully。
    // 触发 write_updated_conversation_state。
    // 从持久化表示恢复 conversation。
    // 断言 text/tool_call/plan/permission 都存在。
}
```

预期：如果当前旧 `Task` protobuf 不保存 ACP-native output，测试失败。

- [x] **Step 2: 选择 ACP-native 持久化结构**

ACP transcript 以 ACP-native messages 为持久化真相。若现有 `AIConversation` 的 `Task` source 无法表达 ACP message，就在 `AgentConversationData` 里增加 ACP transcript JSON 字段，按 conversation id 保存 ACP output；旧 `Task` output 不再作为 ACP transcript 来源。

- [x] **Step 3: 实现最小持久化**

实现要求：

- 保存 ACP assistant text。
- 保存 ACP reasoning text。
- 保存 ACP tool call 的协议字段：`id`、`title`、`kind`、`status`、`content`、`locations`、`raw_input`、`raw_output`。
- 保存 ACP plan entries。
- 保存 ACP permission request 和 selected option。
- 不保存协议外推断结果。

- [x] **Step 4: 运行验证**

```bash
cargo test -p warp --lib ai::blocklist::history_model_test::acp_output_messages_are_available_after_conversation_persistence_roundtrip --locked
cargo test -p warp --lib ai::agent::conversation_tests --locked
```

预期：通过。

---

### Task 4: ACP-aware shortcuts 帮助面板

**文件：**
- 修改：`app/src/ai/blocklist/agent_view/shortcuts/mod.rs`
- 修改：`app/src/terminal/input/agent.rs`
- 测试：`app/src/ai/blocklist/agent_view/shortcuts/model.rs` 或新增同文件单测

- [x] **Step 1: 简化 context**

把 `AgentShortcutsViewContext` 改为：

```rust
pub struct AgentShortcutsViewContext {
    pub has_submitted_first_prompt: bool,
}
```

- [x] **Step 2: 保留通用快捷项**

AgentView 下继续显示：

- `!` input shell command
- `/` for slash commands
- `@` for file paths and attaching other context
- code review
- conversation list / search and continue conversations
- start a new conversation
- `ctrl-c` pause agent
- `escape` go back to terminal

- [x] **Step 3: 移除 ACP 不适用项**

AgentView 下不显示：

- `toggle auto-accept`，除非 ACP permission policy 后续有明确等价设置。
- cloud-only zero state 或 handoff 文案。
- remote-control / shared-session / plugin notifications 相关文案。

- [x] **Step 4: 运行测试**

```bash
cargo test -p warp --lib ai::blocklist::agent_view::shortcuts --locked
```

预期：ACP context 下帮助内容只包含通用项和 ACP 可用项。

---

### Task 5: ACP slash commands 精确对齐

**文件：**
- 修改：`app/src/terminal/input/slash_commands/data_source/mod.rs`
- 修改：`app/src/terminal/input/slash_commands/mod.rs`
- 测试：`app/src/terminal/input/slash_commands/data_source/mod_test.rs`
- 测试：`app/src/terminal/input/slash_command_model_tests.rs`

- [x] **Step 1: 补 available commands 数据源测试**

测试：

```rust
#[test]
fn acp_available_commands_are_visible_only_for_active_acp_conversation() {
    // conversation A 收到 AvailableCommandsUpdate。
    // conversation B 没有。
    // slash command query 在 A 中包含 ACP command，在 B 中不包含。
}
```

- [x] **Step 2: 补 command input hint 测试**

测试：

```rust
#[test]
fn selecting_acp_command_with_input_hint_keeps_editor_open() {
    // AvailableCommandInput::Unstructured("optional task")
    // 选择后 editor buffer 是 "/command "，不立即提交。
}
```

- [x] **Step 3: 补无 input hint 命令提交测试**

测试：

```rust
#[test]
fn selecting_acp_command_without_input_hint_submits_command() {
    // 无 input hint。
    // 选择后进入 ACP send_user_query_in_conversation 路径。
}
```

- [x] **Step 4: 运行验证**

```bash
cargo test -p warp --lib terminal::input::slash_commands --locked
cargo test -p warp --lib terminal::input::slash_command_model_tests --locked
```

预期：通过。

---

### Task 6: ACP context 输入不丢失

**文件：**
- 修改：`app/src/ai/blocklist/controller.rs`
- 修改：`app/src/ai/acp/model.rs`
- 测试：`app/src/ai/blocklist/controller` 相关测试

- [x] **Step 1: 写失败测试，覆盖 attached blocks / files / selected text**

测试目标：

```rust
#[test]
fn acp_prompt_preserves_user_context_inputs() {
    // RequestInput 包含 UserQuery、attached block、selected text 或文件 context。
    // 调用 acp_prompt_from_request。
    // 断言 prompt 中包含用户 query 和 context 摘要。
}
```

- [x] **Step 2: 明确 ACP prompt 构造规则**

最小规则：

- user query 原文保持在最前。
- attached terminal blocks 以文本摘要追加。
- selected text 以文本摘要追加。
- 文件 attachment 使用 ACP `ContentBlock` 表达；如果当前协议类型只能承载文本，则生成包含文件路径和已读取内容摘要的 text block。

不能只 `filter_map(|input| input.user_query())` 后丢弃其他输入。

- [x] **Step 3: 实现 `AcpPromptPayload`**

在 controller 内部先构造轻量结构：

```rust
struct AcpPromptPayload {
    content_blocks: Vec<agent_client_protocol::schema::ContentBlock>,
    display_prompt: String,
}
```

如果暂时只发送 text block，也要让 `display_prompt` 和发送内容一致。

- [x] **Step 4: 运行验证**

```bash
cargo test -p warp --lib ai::blocklist::controller --locked
```

预期：通过。

---

### Task 7: ACP tool call UI 精确映射

**文件：**
- 修改：`app/src/ai/blocklist/block/view_impl/output.rs`
- 修改：`app/src/ai/acp/thread.rs`
- 测试：`app/src/ai/blocklist/block/view_impl/output_tests.rs`
- 测试：`app/src/ai/acp/tests.rs`

- [x] **Step 1: 保留 read skill 精确规则**

规则保持：

- `ToolKind::Read`
- `locations` 指向已识别 skill 的精确 `SKILL.md`
- `location.path == skill_path_from_file_path(location.path)`

不允许从 title、raw input、meta 或 adapter 文案推断。

- [x] **Step 2: 为 diff、terminal、resource、image 补测试**

测试：

```rust
#[test]
fn acp_tool_call_renders_diff_content_as_code_diff_view() {}

#[test]
fn acp_tool_call_renders_terminal_content_from_protocol_terminal_reference() {}

#[test]
fn acp_tool_call_renders_resource_links_without_guessing_kind() {}

#[test]
fn acp_tool_call_decodes_image_content_once() {}
```

- [x] **Step 3: 删除 meta 解析 terminal trace**

删除 `terminal_info`、`terminal_output`、`terminal_exit` 这类 meta 到 terminal trace 的解析。terminal trace 只在 `AcpTerminalManager` 收到 ACP terminal request 后产生，并且只通过 `ToolCallContent::Terminal(terminal_id)` 关联到对应 `AcpToolCall`。

- [x] **Step 4: 运行验证**

```bash
cargo test -p warp --lib ai::blocklist::block::view_impl::output::tests --locked
cargo test -p warp --lib ai::acp --locked
```

预期：通过。

- [x] **Step 5: ACP plan / permission 精确卡片化**

`AcpPlan` 和 `AcpPermission` 不再走通用 `RenderableAction` 文本卡片。`AcpPlan` 映射到 ACP plan card，按 `PlanEntryStatus` 渲染每条计划项；`AcpPermission` 映射到 ACP permission card，按 `RequestPermissionRequest.options` 渲染协议按钮，并保留选择后的完成态。

- [x] **Step 6: 运行验证**

```bash
cargo test -p warp --lib ai::blocklist::block::view_impl::output::tests --locked
cargo test -p warp --lib ai::acp --locked
cargo check -p warp --lib --locked
```

预期：通过。

---

### Task 8: ACP config/current mode UI

**文件：**
- 修改：`app/src/ai/acp/model.rs`
- 修改：`app/src/settings_view/ai_page.rs`
- 可能修改：`app/src/ai/blocklist/agent_view/agent_input_footer/mod.rs`
- 测试：`app/src/settings_view/mod_test.rs`
- 测试：`app/src/settings/ai_tests.rs`

- [x] **Step 1: 明确设置页职责**

设置页只负责：

- 选择 ACP backend。
- 展示该 backend probe 到的 config options。
- 允许用户为 config option 保存默认值。

设置页不硬编码 model/effort 字段，因为 ACP adapter 的 config options 才是来源。

- [x] **Step 2: 写默认值下拉测试**

测试：

```rust
#[test]
fn acp_config_dropdown_selects_current_option_without_default_item() {
    // 有 current_value 时，dropdown 选中对应 option。
    // 没有额外 "Default" option。
}
```

- [x] **Step 3: 当前 conversation mode 显示**

如果 ACP adapter 发送 `CurrentModeUpdate`，在 AgentView footer 或 message bar 以现有 Warp chip 风格显示当前 mode。没有 update 时不显示任何 mode chip。

- [x] **Step 4: 运行验证**

```bash
cargo test -p warp --lib settings::ai_tests --locked
cargo test -p warp --lib settings_view::tests --locked
cargo test -p warp --lib settings_view::ai_page::tests --locked
cargo test -p warp --lib ai::blocklist::agent_view::agent_input_footer::tests --locked
```

预期：通过。

---

### Task 9: 清理旧 Warp-only Agent 控件和命名

**文件：**
- 修改：`app/src/ai/blocklist/agent_view/agent_input_footer/toolbar_item.rs`
- 修改：`app/src/ai/blocklist/agent_view/agent_input_footer/mod.rs`
- 修改：`app/src/ai/acp/model.rs`
- 修改：`app/src/ai/agent/conversation.rs`
- 测试：相关现有 toolbar/settings/acp 测试

- [x] **Step 1: toolbar 默认项继续不包含旧 Warp-only 控件**

保留现有测试：

```rust
agent_view_toolbar_defaults_do_not_include_legacy_warp_agent_controls
agent_view_toolbar_configurator_does_not_offer_legacy_warp_agent_controls
```

旧 Warp-only 控件包括：

- model selector
- context window usage
- remote-control / share session
- fast-forward auto-approve
- handoff to cloud
- voice input

- [x] **Step 2: ACP 命名去 Local**

把概念命名统一成 `Acp*` / `acp-*`。message id 和测试快照一起直接迁移到新命名。

- [x] **Step 3: 删除不准确测试**

删除或改写 “ACP 隐藏 code review” 相关测试。保留 “旧 Warp-only 控件不进入 ACP toolbar” 测试。

- [x] **Step 4: 运行验证**

```bash
cargo test -p warp --lib ai::acp --locked
cargo test -p warp --lib ai::blocklist::agent_view --locked
```

预期：通过。

---

### Task 10: 终端 suggestions 保持独立

**文件：**
- 修改：`app/src/ai/local_suggestions/client.rs`
- 修改：`app/src/ai/local_suggestions/provider.rs`
- 修改：`app/src/ai/predict/next_command_model.rs`
- 修改：`app/src/ai/blocklist/passive_suggestions/legacy.rs`
- 测试：`app/src/ai/local_suggestions/tests.rs`
- 测试：`app/src/ai/predict/next_command_model.rs` 相关测试

- [x] **Step 1: 确认结构化 JSON 解析**

当前 suggestions provider 用 OpenAI-compatible chat completion，并要求模型返回 JSON 字符串。保留 `parse_json_object`，补测试覆盖 markdown fence、非 object、HTTP error。

- [x] **Step 2: effort 不做 OpenAI-only 假设**

`reasoning_effort` 只在非 Default 时发送字符串；Default 不发送。文案保持 “provider-specific / optional”。

- [x] **Step 3: Next Command 和 Prompt Suggestions 开关独立**

保留：

- `local_next_command_enabled`
- `local_prompt_suggestions_enabled`

它们不依赖 ACP backend，也不受 AgentView 是否 ACP conversation 影响。

- [x] **Step 4: 运行验证**

```bash
cargo test -p warp --lib ai::local_suggestions --locked
cargo test -p warp --lib ai::predict::next_command_model --locked
```

预期：通过。

---

### Task 11: 手动 UI 验证

**文件：**
- 不改文件

- [x] **Step 1: 编译**

```bash
cargo check -p warp --lib --locked
```

预期：通过，最多只有已有 warning。

- [x] **Step 2: 格式检查**

```bash
cargo fmt --all --check
git diff --check
```

预期：通过。

- [ ] **Step 3: 运行应用后验证 AgentView**

手动验证：

- 输入中文自然语言，例如 `你好`，触发 ACP。
- 输入 `/agent 你好`，触发同一 ACP 流程。
- assistant thought 和 assistant final 分开显示；如果 adapter 只发 thought，那 UI 只显示 thought，不做伪造 final。
- read skill 使用旧 Warp read skill 行组件。
- tool call diff 显示为 Warp diff UI。
- permission request 可以选择并回写 adapter。
- `/` 显示 ACP available commands。
- `?` 只显示通用和 ACP 可用快捷键。
- `⌘Y` 能打开/搜索 conversation，并能继续已有 ACP conversation。
- `⇧⌘+` 能打开右侧 code review/diff 面板。

- [ ] **Step 4: 重启验证 conversation**

手动验证：

- 完成一次 ACP 对话。
- 重启应用。
- 打开 conversation 菜单。
- 能看到历史 ACP conversation。
- 打开历史 ACP conversation 后，assistant text、thought、tool call、plan、permission 仍能显示。

---

### Task 12: 基于旧链路恢复 ACP conversation 入口

**文件：**
- 修改：`app/src/workspace/view.rs`
- 修改：`app/src/workspace/mod.rs`
- 修改：`app/src/ai/agent_conversations_model.rs`
- 测试：`app/src/workspace/view_test.rs`
- 测试：`app/src/terminal/input_test.rs`
- 测试：`app/src/ai/agent_conversations_model_tests.rs`

- [x] **Step 1: 恢复 tools panel conversation 图标**

`Workspace::compute_left_panel_views` 重新把 `ToolPanelView::ConversationListView` 纳入可用 views，并恢复 `SHOW_CONVERSATION_HISTORY` keymap context。

- [x] **Step 2: 恢复 conversation list keybindings 和 action**

恢复 `LEFT_PANEL_AGENT_CONVERSATIONS_BINDING_NAME`、`TOGGLE_CONVERSATION_LIST_VIEW_BINDING_NAME` 绑定；`WorkspaceAction::ToggleConversationListView` 重新打开旧 `LeftPanelAction::ConversationListView`。

- [x] **Step 3: 恢复左侧列表的新建 conversation 行为**

`LeftPanelEvent::NewConversationInNewTab` 重新创建 terminal tab，并用 `AgentViewEntryOrigin::ConversationListView` 进入新 ACP AgentView conversation。

- [x] **Step 4: 接通本地 history 到 conversation list model**

`AgentConversationsModel` 订阅 `BlocklistAIHistoryModel`，让 ACP 写入的 `AIConversation` 继续进入旧 conversation list 数据流。

---

## 最终验证命令

```bash
cargo test -p warp --lib ai::acp --locked
cargo test -p warp --lib ai::blocklist::block::view_impl::output::tests --locked
cargo test -p warp --lib ai::blocklist::agent_view --locked
cargo test -p warp --lib terminal::input::slash_commands --locked
cargo test -p warp --lib ai::local_suggestions --locked
cargo check -p warp --lib --locked
cargo fmt --all --check
git diff --check
```

---

## 自检

- 覆盖旧 Warp AI 入口：已覆盖 NLD、`/agent`、AgentView、message bar、shortcuts、toolbar、conversation、code review、slash commands、context、suggestions。
- 覆盖 ACP 事件：已覆盖 text、thought、tool call、tool update、terminal trace、plan、available commands、current mode、config options、session info、permission、completion/error。
- 没有协议外补造：read skill、tool kind、locations、content、terminal trace 渲染均要求协议字段或 ACP terminal request 事件链。
- 不误删通用 UI：code review 和 conversation navigation 不按 ACP 后端隐藏。
- 不迁移云端能力：remote-control、handoff、shared session、Warp Drive/Teams/Login 不进入 ACP 单机路径。
